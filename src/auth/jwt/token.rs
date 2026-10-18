use base64::prelude::*;
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::error::Error;

const ACCESS_KEY: &str = "ACCESS";
const REFRESH_KEY: &str = "REFRESH";

#[derive(Serialize, Deserialize)]
pub struct JWT<T> {
    data: T,
}

pub enum JWTType {
    Refresh,
    Access,
}

type HmacSha256 = Hmac<Sha256>;

impl<T> JWT<T> {
    pub fn new(data: T) -> Self {
        JWT { data }
    }
}

//implement serialize for JWT.
impl<T: serde::Serialize> JWT<T> {
    pub fn to_jwt_string(&self, typ: JWTType) -> Result<String, Box<dyn Error>> {
        let json = serde_json::to_string(&self.data)?;
        let base64 = BASE64_STANDARD.encode(json.as_bytes());

        let key = match typ {
            JWTType::Access => ACCESS_KEY,
            JWTType::Refresh => REFRESH_KEY,
        };

        let mut mac = HmacSha256::new_from_slice(key.as_bytes())?;
        mac.update(json.as_bytes());

        let signature = BASE64_URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());

        Ok(format!("{signature}.{base64}"))
    }
}

impl<T: serde::de::DeserializeOwned> JWT<T> {
    pub fn from_string_to_jwt(input: &str, typ: JWTType) -> Result<JWT<T>, Box<dyn Error>> {
        let (signature, payload) = input
            .split_once('.')
            .ok_or("payload is not valid jwt for the service")?;

        if payload.contains('.') {
            return Err("Payload still contains '.'".into());
        }

        let key = match typ {
            JWTType::Access => ACCESS_KEY,
            JWTType::Refresh => REFRESH_KEY,
        };

        let signature = BASE64_URL_SAFE_NO_PAD.decode(signature)?;
        let json = BASE64_STANDARD.decode(payload)?;

        let mut mac = HmacSha256::new_from_slice(key.as_bytes())?;
        mac.update(&json);
        mac.verify_slice(&signature)?;

        let data = serde_json::from_slice(&json)?;
        Ok(JWT { data })
    }
}
