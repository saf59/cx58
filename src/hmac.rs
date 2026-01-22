use sha2::Sha256;
use hmac::{Hmac, Mac};
use hmac::digest::InvalidLength;

type HmacSha256 = Hmac<Sha256>;

pub fn build_hmac(secret: &str, payload: &[u8] ) -> Result<(i64, String),InvalidLength> {
    let timestamp = chrono::Utc::now().timestamp();
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;
    mac.update(timestamp.to_string().as_bytes());
    mac.update(payload);
    let signature = hex::encode(mac.finalize().into_bytes());
    Ok((timestamp, signature))
}
