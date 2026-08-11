use anyhow::{Result, bail};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn build_hmac(secret: &str, payload: &[u8]) -> Result<(i64, String)> {
    let timestamp = chrono::Utc::now().timestamp();
    let signature = build_hmac_at(secret, payload, timestamp)?;
    Ok((timestamp, signature))
}

fn build_hmac_at(secret: &str, payload: &[u8], timestamp: i64) -> Result<String> {
    if secret.trim().is_empty() {
        bail!("HMAC secret must not be empty");
    }

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;
    mac.update(timestamp.to_string().as_bytes());
    mac.update(payload);
    Ok(hex::encode(mac.finalize().into_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_depends_on_exact_payload_bytes() {
        let timestamp = 1_700_000_000;
        let first = build_hmac_at("demo-secret", br#"{"message":"hello"}"#, timestamp).unwrap();
        let second = build_hmac_at("demo-secret", br#"{"message": "hello"}"#, timestamp).unwrap();

        assert_ne!(first, second);
    }

    #[test]
    fn empty_secret_is_rejected() {
        assert!(build_hmac("  ", b"payload").is_err());
    }
}
