use std::env;

#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub sender_email: String,
    pub sender_name: String,
    pub frontend_url: String,
}

impl EmailConfig {
    pub fn from_env() -> Self {
        Self {
            smtp_host: env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string()),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .expect("SMTP_PORT must be a number"),
            smtp_username: env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set"),
            smtp_password: env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set"),
            sender_email: env::var("SENDER_EMAIL")
                .unwrap_or_else(|_| "noreply@safatanc-connect.com".to_string()),
            sender_name: env::var("SENDER_NAME").unwrap_or_else(|_| "Safatanc Connect".to_string()),
            frontend_url: env::var("FRONTEND_URL").expect("FRONTEND_URL must be set"),
        }
    }
}
