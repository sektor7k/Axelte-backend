
use chrono::Utc;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};


const SECRET_KEY: &str = "secret";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn generate_token(user_id: &str) -> String {
    let exp = Utc::now()
        .checked_add_signed(chrono::Duration::days(1))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims { sub: user_id.to_string(), exp };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET_KEY.as_ref()),
    ).unwrap()
}

pub fn verify_token(token: &str) -> jsonwebtoken::errors::Result<jsonwebtoken::TokenData<Claims>> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET_KEY.as_ref()),
        &Validation::default(),
    )
}


