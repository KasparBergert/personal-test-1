use base64::prelude::*;
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::auth::jwt::{ACCESS_KEY, REFRESH_KEY};

#[derive(Serialize, Deserialize)]
pub struct JWT<T> {
    pub data: T,
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

#[derive(Debug)]
pub enum JWTError {
    InvalidFormat,
    InvalidSignatureEncoding,
    SerializationFailed,
    InvalidPayloadEncoding,
    InvalidKey,
    InvalidSignature,
    InvalidPayload,
}

impl JWTError {
    pub fn status_code(&self) -> actix_web::http::StatusCode {
        use actix_web::http::StatusCode;

        match self {
            Self::InvalidFormat
            | Self::InvalidSignatureEncoding
            | Self::InvalidPayloadEncoding
            | Self::InvalidPayload => StatusCode::BAD_REQUEST,

            Self::InvalidSignature => StatusCode::UNAUTHORIZED,

            Self::SerializationFailed | Self::InvalidKey => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::fmt::Display for JWTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat => write!(f, "JWT must contain a signature and payload"),
            Self::InvalidSignatureEncoding => write!(f, "Invalid signature encoding"),
            Self::SerializationFailed => write!(f, "String failed to serialize into json"),
            Self::InvalidPayloadEncoding => write!(f, "Invalid payload encoding"),
            Self::InvalidKey => write!(f, "Invalid signing key"),
            Self::InvalidSignature => write!(f, "Signature verification failed"),
            Self::InvalidPayload => write!(f, "Invalid JWT payload"),
        }
    }
}

impl std::error::Error for JWTError {}

//implement serialize for JWT.
impl<T: serde::Serialize> JWT<T> {
    /** creates jwt into a string */
    pub fn to_jwt_string(&self, typ: JWTType) -> Result<String, JWTError> {
        let json = serde_json::to_string(&self.data).map_err(|_| JWTError::SerializationFailed)?;

        let json_base64 = BASE64_STANDARD.encode(json.as_bytes());

        let key = match typ {
            JWTType::Access => ACCESS_KEY,
            JWTType::Refresh => REFRESH_KEY,
        };

        let mut mac =
            HmacSha256::new_from_slice(key.as_bytes()).map_err(|_| JWTError::InvalidKey)?;
        mac.update(json.as_bytes());

        let signature = BASE64_URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());

        Ok(format!("{signature}.{json_base64}"))
    }
}

impl<T: serde::de::DeserializeOwned> JWT<T> {
    /** validates and returns contents of jwt */
    pub fn from_string_to_jwt(input: &str, typ: JWTType) -> Result<Self, JWTError> {
        let (signature, payload) = input.split_once('.').ok_or(JWTError::InvalidFormat)?;

        if payload.contains('.') {
            return Err(JWTError::InvalidFormat);
        }

        let key = match typ {
            JWTType::Access => ACCESS_KEY,
            JWTType::Refresh => REFRESH_KEY,
        };

        let signature = BASE64_URL_SAFE_NO_PAD
            .decode(signature)
            .map_err(|_| JWTError::InvalidSignatureEncoding)?;

        let json = BASE64_STANDARD
            .decode(payload)
            .map_err(|_| JWTError::InvalidPayloadEncoding)?;

        let mut mac =
            HmacSha256::new_from_slice(key.as_bytes()).map_err(|_| JWTError::InvalidKey)?;
        mac.update(&json);
        mac.verify_slice(&signature)
            .map_err(|_| JWTError::InvalidSignature)?;

        let data = serde_json::from_slice(&json).map_err(|_| JWTError::InvalidPayload)?;

        Ok(JWT { data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestPayload {
        user_id: u32,
    }

    #[test]
    fn jwt_string_can_be_decoded() {
        let original = JWT {
            data: TestPayload { user_id: 42 },
        };

        let encoded = original.to_jwt_string(JWTType::Access).unwrap();
        let decoded = JWT::<TestPayload>::from_string_to_jwt(&encoded, JWTType::Access).unwrap();

        assert_eq!(decoded.data, original.data);
    }
}
