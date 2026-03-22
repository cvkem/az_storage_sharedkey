use base64::prelude::*;
use hmac::{Hmac, Mac};
use sha2::Sha256;

// Define HMAC type
pub type HmacSha256 = Hmac<Sha256>;

/// base64 decoding for utf8 strings
pub fn b64_decode_utf8(encoded_data: &str) -> String {
    //let encoded_data = "SGVsbG8sIFJ1c3Qh"; // "Hello, Rust!"

    match BASE64_STANDARD.decode(encoded_data) {
        Ok(decoded_bytes) => {
            // Convert bytes to a string, handling potential UTF-8 errors gracefully
            let decoded_string = String::from_utf8_lossy(&decoded_bytes);
            // println!("Decoded: {}", decoded_string); // Output: Decoded: Hello, Rust!
            decoded_string.to_string()
        }
        Err(e) => {
            eprintln!("Decoding error: {}", e);
            "".to_owned()
        }
    }
}

/// decoding base64 for binary data.
pub fn b64_decode(encoded_data: &str) -> Vec<u8> {
    BASE64_STANDARD
        .decode(encoded_data)
        .expect("Invalid base64 encoding (based on BASE64_STANDARD engine)")
}

pub fn get_hmac_b64(key_b64: &str, message: &str) -> String {
    // extract the base-64 encoded key
    let key = b64_decode(key_b64);

    // The message to authenticate
    let message_bytes = message.as_bytes();
    // Create HMAC object
    let mut hmac = HmacSha256::new_from_slice(key.as_slice()).expect("Invalid key length");
    // Input message
    hmac.update(message_bytes);
    // Obtain the result
    let result = hmac.finalize();

    let code_bytes = result.into_bytes();
    BASE64_STANDARD.encode(code_bytes)
}
