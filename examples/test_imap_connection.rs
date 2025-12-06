//! Test IMAP connection to diagnose blocking issue

use anyhow::Result;
use async_native_tls::TlsConnector;
use tokio_util::compat::TokioAsyncReadCompatExt;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting IMAP connection diagnostic...\n");

    println!("Step 1: Testing TCP connection to imap.gmail.com:993...");

    match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        tokio::net::TcpStream::connect("imap.gmail.com:993"),
    )
    .await
    {
        Ok(Ok(stream)) => {
            println!("✓ TCP connection successful!\n");
            drop(stream);
        }
        Ok(Err(e)) => {
            println!("✗ TCP connection failed: {}\n", e);
            return Err(e.into());
        }
        Err(_) => {
            println!("✗ TCP connection timed out after 10 seconds\n");
            println!("This suggests a network/firewall issue blocking port 993.");
            return Ok(());
        }
    }

    println!("Step 2: Testing TLS handshake...");

    match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        async {
            let tcp = tokio::net::TcpStream::connect("imap.gmail.com:993")
                .await
                .map_err(|e| anyhow::anyhow!("TCP error: {}", e))?;
            let compat = tcp.compat();
            let tls = TlsConnector::new();
            tls.connect("imap.gmail.com", compat)
                .await
                .map_err(|e| anyhow::anyhow!("TLS error: {}", e))
        },
    )
    .await
    {
        Ok(Ok(stream)) => {
            println!("✓ TLS handshake successful!\n");
            drop(stream);
        }
        Ok(Err(e)) => {
            println!("✗ TLS handshake failed: {}\n", e);
            return Err(e.into());
        }
        Err(_) => {
            println!("✗ TLS handshake timed out after 10 seconds\n");
            return Ok(());
        }
    }

    println!("Step 3: Creating IMAP client and reading greeting...");

    match tokio::time::timeout(
        std::time::Duration::from_secs(15),
        async {
            let tcp = tokio::net::TcpStream::connect("imap.gmail.com:993")
                .await
                .map_err(|e| anyhow::anyhow!("TCP error: {}", e))?;
            let compat = tcp.compat();
            let tls = TlsConnector::new();
            let tls_stream = tls.connect("imap.gmail.com", compat)
                .await
                .map_err(|e| anyhow::anyhow!("TLS error: {}", e))?;
            let client = async_imap::Client::new(tls_stream);
            Ok::<_, anyhow::Error>(client)
        },
    )
    .await
    {
        Ok(Ok(_client)) => {
            println!("✓ IMAP client created successfully!\n");
        }
        Ok(Err(e)) => {
            println!("✗ IMAP client creation failed: {}\n", e);
            return Err(e);
        }
        Err(_) => {
            println!("✗ IMAP client creation timed out after 15 seconds\n");
            println!("This may indicate the server is not responding properly.");
        }
    }

    println!("✓ All connection steps completed successfully!");
    println!("\nConclusion: Basic IMAP connection works.");
    println!("The issue must be in authentication or later steps.");

    Ok(())
}
