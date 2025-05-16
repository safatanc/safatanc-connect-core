use std::env;

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    // Google OAuth
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_auth_url: String,
    pub google_token_url: String,
    pub google_redirect_url: String,
    pub google_user_info_url: String,

    // GitHub OAuth
    pub github_client_id: String,
    pub github_client_secret: String,
    pub github_auth_url: String,
    pub github_token_url: String,
    pub github_redirect_url: String,
    pub github_user_info_url: String,
}

impl OAuthConfig {
    pub fn from_env() -> Self {
        Self {
            // Google OAuth config
            google_client_id: env::var("OAUTH_GOOGLE_CLIENT_ID")
                .unwrap_or_else(|_| "your-google-client-id.apps.googleusercontent.com".to_string()),
            google_client_secret: env::var("OAUTH_GOOGLE_CLIENT_SECRET")
                .unwrap_or_else(|_| "your-google-client-secret".to_string()),
            google_auth_url: env::var("OAUTH_GOOGLE_AUTH_URL")
                .unwrap_or_else(|_| "https://accounts.google.com/o/oauth2/v2/auth".to_string()),
            google_token_url: env::var("OAUTH_GOOGLE_TOKEN_URL")
                .unwrap_or_else(|_| "https://oauth2.googleapis.com/token".to_string()),
            google_redirect_url: env::var("OAUTH_GOOGLE_REDIRECT_URL").unwrap_or_else(|_| {
                "http://localhost:8080/api/auth/oauth/google/callback".to_string()
            }),
            google_user_info_url: env::var("OAUTH_GOOGLE_USER_INFO_URL")
                .unwrap_or_else(|_| "https://www.googleapis.com/oauth2/v2/userinfo".to_string()),

            // GitHub OAuth config
            github_client_id: env::var("OAUTH_GITHUB_CLIENT_ID")
                .unwrap_or_else(|_| "your-github-client-id".to_string()),
            github_client_secret: env::var("OAUTH_GITHUB_CLIENT_SECRET")
                .unwrap_or_else(|_| "your-github-client-secret".to_string()),
            github_auth_url: env::var("OAUTH_GITHUB_AUTH_URL")
                .unwrap_or_else(|_| "https://github.com/login/oauth/authorize".to_string()),
            github_token_url: env::var("OAUTH_GITHUB_TOKEN_URL")
                .unwrap_or_else(|_| "https://github.com/login/oauth/access_token".to_string()),
            github_redirect_url: env::var("OAUTH_GITHUB_REDIRECT_URL").unwrap_or_else(|_| {
                "http://localhost:8080/api/auth/oauth/github/callback".to_string()
            }),
            github_user_info_url: env::var("OAUTH_GITHUB_USER_INFO_URL")
                .unwrap_or_else(|_| "https://api.github.com/user".to_string()),
        }
    }
}
