use crate::util::{config::CONFIG, errors::ApiError};

use actix_identity::{CookieIdentityPolicy, IdentityService};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub discord: DiscordUser,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiscordUser {
	pub id: String,
	pub username: String,
	pub discriminator: String,
	pub avatar: Option<String>,
	pub bot: Option<bool>,
	pub sytem: Option<bool>,
	pub mfa_enabled: Option<bool>,
	pub locale: Option<String>,
	pub verified: Option<bool>,
	pub email: Option<String>,
	pub flags: Option<u128>,
	pub premium_type: Option<u8>,
	pub public_flags: Option<u128>
}

impl DiscordUser {
    pub fn tag(&self) -> String {
        format!("{}#{}", self.username, self.discriminator)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivateClaim {
    pub id: String,
    pub discord: DiscordUser,
    exp: i64,
}

impl PrivateClaim {
    pub fn new(discord: DiscordUser) -> Self {
        Self {
            id: discord.id.clone(),
            discord,
            exp: (Utc::now() + Duration::hours(CONFIG.jwt_expiration)).timestamp(),
        }
    }
}

/// Create a json web token (JWT)
pub fn create_jwt(private_claim: PrivateClaim) -> Result<String, ApiError> {
    let encoding_key = EncodingKey::from_secret(&CONFIG.jwt_key.as_ref());
    encode(&Header::default(), &private_claim, &encoding_key)
        .map_err(|e| ApiError::CannotEncodeJwtToken(e.to_string()))
}

/// Decode a json web token (JWT)
pub fn decode_jwt(token: &str) -> Result<PrivateClaim, ApiError> {
    let decoding_key = DecodingKey::from_secret(&CONFIG.jwt_key.as_ref());
    decode::<PrivateClaim>(token, &decoding_key, &Validation::default())
        .map(|data| data.claims)
        .map_err(|e| ApiError::CannotDecodeJwtToken(e.to_string()))
}

/// Gets the identidy service for injection into an Actix app
pub fn get_identity_service() -> IdentityService<CookieIdentityPolicy> {
    IdentityService::new(
        CookieIdentityPolicy::new(&CONFIG.session_key.as_ref())
            .name(&CONFIG.session_name)
            .max_age_time(chrono::Duration::minutes(CONFIG.session_timeout))
            .secure(CONFIG.session_secure),
    )
}
