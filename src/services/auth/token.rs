use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::errors::AppError;
use crate::models::user::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,   // Subject (user ID)
    pub exp: i64,      // Expiration time
    pub iat: i64,      // Issued at
    pub email: String, // User email
    pub role: String,  // User role
}

pub struct TokenService {
    config: AppConfig,
}

impl TokenService {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    // Generate token and refresh token for user
    pub fn generate_tokens(&self, user: &User) -> Result<(String, String), AppError> {
        let now = Utc::now();
        let token_exp = now + Duration::seconds(self.config.jwt_expiration);
        let refresh_token_exp = now + Duration::seconds(self.config.refresh_token_expiration);

        // Claims for the main token
        let claims = Claims {
            sub: user.id.to_string(),
            exp: token_exp.timestamp(),
            iat: now.timestamp(),
            email: user.email.clone(),
            role: user.global_role.clone(),
        };

        // Claims for refresh token (same, but with different expiry)
        let refresh_claims = Claims {
            sub: user.id.to_string(),
            exp: refresh_token_exp.timestamp(),
            iat: now.timestamp(),
            email: user.email.clone(),
            role: user.global_role.clone(),
        };

        // Encode token
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("Failed to generate token: {}", e)))?;

        // Encode refresh token
        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("Failed to generate refresh token: {}", e)))?;

        Ok((token, refresh_token))
    }

    // Verify token and return claims
    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        let decoded = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                AppError::Authentication("Token has expired".into())
            }
            _ => AppError::Authentication("Invalid token".into()),
        })?;

        Ok(decoded.claims)
    }

    // Refresh token to get a new token
    pub fn refresh_token(&self, refresh_token: &str) -> Result<String, AppError> {
        let claims = self.verify_token(refresh_token)?;

        // Create a new token with same claims but new expiry
        let now = Utc::now();
        let token_exp = now + Duration::seconds(self.config.jwt_expiration);

        let new_claims = Claims {
            sub: claims.sub,
            exp: token_exp.timestamp(),
            iat: now.timestamp(),
            email: claims.email,
            role: claims.role,
        };

        let new_token = encode(
            &Header::default(),
            &new_claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("Failed to generate new token: {}", e)))?;

        Ok(new_token)
    }

    // Extract user ID from token
    pub fn get_user_id_from_token(&self, token: &str) -> Result<Uuid, AppError> {
        let claims = self.verify_token(token)?;
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
            AppError::Authentication("Token contains invalid user ID".into())
        })?;

        Ok(user_id)
    }
}
