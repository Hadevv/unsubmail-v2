//! XOAUTH2 SASL authentication for IMAP

#[cfg(test)]
use base64::{engine::general_purpose::STANDARD, Engine};

/// Build XOAUTH2 authentication string (raw, not base64-encoded)
///
/// Format: `user=<email>\x01auth=Bearer <access_token>\x01\x01`
///
/// NOTE: The async-imap crate handles base64 encoding internally,
/// so this function returns the raw (unencoded) authentication string.
///
/// # Example
/// ```
/// use unsubmail::infrastructure::imap::auth::build_xoauth2_string;
///
/// let auth = build_xoauth2_string("user@gmail.com", "ya29.token123");
/// assert!(auth.contains("user=user@gmail.com"));
/// assert!(auth.contains("auth=Bearer ya29.token123"));
/// ```
pub fn build_xoauth2_string(email: &str, access_token: &str) -> String {
    format!(
        "user={}\x01auth=Bearer {}\x01\x01",
        email, access_token
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xoauth2_string() {
        let result = build_xoauth2_string("test@gmail.com", "token123");

        // Verify raw format (not base64-encoded)
        assert!(result.starts_with("user=test@gmail.com"));
        assert!(result.contains("auth=Bearer token123"));
        assert!(result.ends_with("\x01\x01"));

        // Verify control characters are present
        assert!(result.contains('\x01'));
    }

    #[test]
    fn test_xoauth2_base64_encoding() {
        // This test shows what the base64-encoded version looks like
        // (for reference, but async-imap does this internally)
        let raw = build_xoauth2_string("test@gmail.com", "token123");
        let encoded = STANDARD.encode(raw.as_bytes());

        // The encoded version should be longer than the raw version
        assert!(encoded.len() > raw.len());

        // Decode to verify round-trip
        let decoded = STANDARD.decode(&encoded).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();
        assert_eq!(decoded_str, raw);
    }
}
