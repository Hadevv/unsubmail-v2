//! IMAP message fetching and header parsing

use super::connection::ImapSession;
use anyhow::{Context, Result};
use futures::TryStreamExt; // Required for try_next()
use mailparse::{parse_mail, MailHeaderMap};
use rayon::prelude::*;
use std::collections::HashMap;

/// Message header data
#[derive(Debug, Clone)]
pub struct MessageHeader {
    pub uid: u32,
    pub from: String,
    pub subject: String,
    pub list_unsubscribe: Option<String>,
    pub list_unsubscribe_post: Option<String>,
}

/// Search for all message UIDs in INBOX
pub async fn search_all_uids(session: &mut ImapSession) -> Result<Vec<u32>> {
    session
        .select("INBOX")
        .await
        .context("Failed to select INBOX")?;

    let search_result = session
        .uid_search("ALL")
        .await
        .context("Failed to search messages")?;

    Ok(search_result.into_iter().collect())
}

/// Fetch headers for a batch of UIDs
pub async fn fetch_headers_batch(
    session: &mut ImapSession,
    uids: &[u32],
) -> Result<Vec<MessageHeader>> {
    if uids.is_empty() {
        return Ok(vec![]);
    }

    let uid_set = format_uid_set(uids);

    tracing::debug!("Fetching headers for UID set: {}", uid_set);

    let mut messages_stream = session
        .uid_fetch(&uid_set, "BODY.PEEK[HEADER]")
        .await
        .context("Failed to fetch headers")?;

    let mut headers = Vec::new();

    // IMPORTANT: Use try_next() instead of next() to properly handle stream termination
    while let Some(msg) = messages_stream
        .try_next()
        .await
        .context("Error reading from fetch stream")?
    {
        tracing::trace!(
            "Received FETCH response - UID: {:?}, has header: {}, message: {}",
            msg.uid,
            msg.header().is_some(),
            msg.message
        );

        // IMPORTANT: Use msg.header() for BODY.PEEK[HEADER] requests, NOT msg.body()
        if let (Some(uid), Some(header_bytes)) = (msg.uid, msg.header()) {
            match parse_message_header(uid, header_bytes) {
                Ok(header) => {
                    tracing::trace!("Parsed header for UID {}: from={}", uid, header.from);
                    headers.push(header);
                }
                Err(e) => {
                    tracing::warn!("Failed to parse header for UID {}: {}", uid, e);
                }
            }
        } else {
            tracing::warn!(
                "Message missing UID={:?} or header={}",
                msg.uid,
                msg.header().is_some()
            );
        }
    }

    tracing::debug!("Successfully fetched {} headers", headers.len());

    Ok(headers)
}

/// Fetch all headers with batching
pub async fn fetch_all_headers(
    session: &mut ImapSession,
    batch_size: usize,
) -> Result<Vec<MessageHeader>> {
    let uids = search_all_uids(session).await?;

    let mut all_headers = Vec::new();

    for chunk in uids.chunks(batch_size) {
        let headers = fetch_headers_batch(session, chunk).await?;
        all_headers.extend(headers);
    }

    Ok(all_headers)
}

/// Parse message header from raw bytes
fn parse_message_header(uid: u32, raw: &[u8]) -> Result<MessageHeader> {
    let mail = parse_mail(raw).context("Failed to parse email")?;

    let from = mail.headers.get_first_value("From").unwrap_or_default();

    let subject = mail.headers.get_first_value("Subject").unwrap_or_default();

    let list_unsubscribe = mail.headers.get_first_value("List-Unsubscribe");
    let list_unsubscribe_post = mail.headers.get_first_value("List-Unsubscribe-Post");

    Ok(MessageHeader {
        uid,
        from,
        subject,
        list_unsubscribe,
        list_unsubscribe_post,
    })
}

/// Format UIDs for IMAP command (e.g., "1,2,3" or "1:100")
fn format_uid_set(uids: &[u32]) -> String {
    if uids.is_empty() {
        return String::new();
    }

    if uids.len() == 1 {
        return uids[0].to_string();
    }

    // Check if consecutive
    let is_consecutive = uids.windows(2).all(|w| w[1] == w[0] + 1);

    if is_consecutive {
        format!("{}:{}", uids[0], uids[uids.len() - 1])
    } else {
        uids.iter()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}

/// Group headers by sender email
pub fn group_by_sender(headers: Vec<MessageHeader>) -> HashMap<String, Vec<MessageHeader>> {
    headers
        .into_par_iter()
        .fold(HashMap::new, |mut acc, header| {
            let email = extract_email(&header.from);
            acc.entry(email).or_insert_with(Vec::new).push(header);
            acc
        })
        .reduce(HashMap::new, |mut acc, map| {
            for (email, mut msgs) in map {
                acc.entry(email).or_insert_with(Vec::new).append(&mut msgs);
            }
            acc
        })
}

/// Extract email address from From header
///
/// Examples:
/// - "John Doe <john@example.com>" -> "john@example.com"
/// - "john@example.com" -> "john@example.com"
fn extract_email(from: &str) -> String {
    if let Some(start) = from.find('<') {
        if let Some(end) = from.find('>') {
            return from[start + 1..end].to_string();
        }
    }

    from.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uid_set_consecutive() {
        let uids = vec![1, 2, 3, 4, 5];
        assert_eq!(format_uid_set(&uids), "1:5");
    }

    #[test]
    fn test_format_uid_set_non_consecutive() {
        let uids = vec![1, 3, 5, 7];
        assert_eq!(format_uid_set(&uids), "1,3,5,7");
    }

    #[test]
    fn test_extract_email() {
        assert_eq!(
            extract_email("John Doe <john@example.com>"),
            "john@example.com"
        );
        assert_eq!(extract_email("john@example.com"), "john@example.com");
    }
}
