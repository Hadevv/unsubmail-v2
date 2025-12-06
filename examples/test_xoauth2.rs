//! Test XOAUTH2 authentication format

use base64::{engine::general_purpose::STANDARD, Engine};
use unsubmail::infrastructure::imap::auth::build_xoauth2_string;

fn main() {
    let email = "test@gmail.com";
    let token = "ya29.test123";

    let auth_string = build_xoauth2_string(email, token);

    println!("Base64 encoded: {}", auth_string);

    // Decode to verify
    let decoded = STANDARD.decode(&auth_string).unwrap();
    let decoded_str = String::from_utf8(decoded).unwrap();

    println!("Decoded: {:?}", decoded_str);
    println!(
        "\nExpected format: user={}\\x01auth=Bearer {}\\x01\\x01",
        email, token
    );

    // Show byte representation
    println!("\nByte representation:");
    for (i, byte) in decoded_str.as_bytes().iter().enumerate() {
        if *byte == 1 {
            print!("\\x01 ");
        } else if byte.is_ascii_graphic() || *byte == b' ' {
            print!("{} ", *byte as char);
        } else {
            print!("0x{:02x} ", byte);
        }
        if (i + 1) % 20 == 0 {
            println!();
        }
    }
    println!();
}
