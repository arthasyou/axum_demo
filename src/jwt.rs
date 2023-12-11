use axum::http::StatusCode;
use bcrypt::verify;
use chrono::{Duration, Local};
use dotenvy_macro::dotenv;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    // pub aud: String,
    // pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn create_jwt() -> Result<String, StatusCode> {
    let mut now = Local::now();
    let iat = now.timestamp() as usize;
    let expire_in = Duration::seconds(30);
    now += expire_in;
    let exp = now.timestamp() as usize;

    let claims = Claims {
        // aud: todo!(),
        // sub: todo!(),
        exp: exp,
        iat: iat,
    };

    let secret = dotenv!("JWT_SECRETE");
    let key = EncodingKey::from_secret(secret.as_bytes());

    encode(&Header::default(), &claims, &key).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn is_valid(token: &str) -> Result<bool, StatusCode> {
    let secret = dotenv!("JWT_SECRETE");
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = &Validation::new(Algorithm::HS256);
    decode::<Claims>(token, &key, validation).map_err(|_| StatusCode::UNAUTHORIZED)?;
    Ok(true)
}
