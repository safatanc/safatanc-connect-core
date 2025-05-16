use lettre::{
    message::{header::ContentType, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use std::sync::Arc;
use tokio::task;
use uuid::Uuid;

use crate::config::EmailConfig;
use crate::db::repositories::TokenRepository;
use crate::errors::AppError;
use crate::models::auth::token::{CreateVerificationTokenDto, TOKEN_TYPE_EMAIL_VERIFICATION};

pub struct EmailService {
    email_config: EmailConfig,
    token_repo: TokenRepository,
}

impl EmailService {
    pub fn new(email_config: EmailConfig, token_repo: TokenRepository) -> Self {
        Self {
            email_config,
            token_repo,
        }
    }

    // Create SMTP transport
    fn create_transport(&self) -> Result<AsyncSmtpTransport<Tokio1Executor>, AppError> {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&self.email_config.smtp_host)
            .map_err(|e| AppError::Internal(format!("Failed to create SMTP transport: {}", e)))?
            .port(self.email_config.smtp_port)
            .credentials(Credentials::new(
                self.email_config.smtp_username.clone(),
                self.email_config.smtp_password.clone(),
            ))
            .build();

        Ok(transport)
    }

    // Send verification email to user
    pub async fn send_verification_email(
        &self,
        user_id: Uuid,
        email: &str,
        username: &str,
    ) -> Result<(), AppError> {
        // Generate verification token
        let token = self.generate_verification_token(user_id).await?;

        // Create verification URL
        let verification_url = format!(
            "{}/auth/verify-email/{}",
            self.email_config.frontend_url, token
        );

        // Email subject
        let subject = "Verify Your Email Address";

        // Build HTML and text content
        let html_content = format!(
            r#"
            <html>
                <body>
                    <h1>Email Verification</h1>
                    <p>Hello {username},</p>
                    <p>Thank you for registering with Safatanc Connect. 
                    Please click the button below to verify your email address:</p>
                    <p>
                        <a href="{verification_url}" style="padding: 10px 15px; background-color: #4CAF50; color: white; text-decoration: none; border-radius: 5px;">
                            Verify Email
                        </a>
                    </p>
                    <p>If the button doesn't work, you can also copy and paste the following link into your browser:</p>
                    <p>{verification_url}</p>
                    <p>This link will expire in 24 hours.</p>
                    <p>If you didn't create an account, you can safely ignore this email.</p>
                    <p>Best regards,<br/>Safatanc Connect Team</p>
                </body>
            </html>
            "#,
            username = username,
            verification_url = verification_url
        );

        let text_content = format!(
            r#"Email Verification

Hello {username},

Thank you for registering with Safatanc Connect. Please use the link below to verify your email address:

{verification_url}

This link will expire in 24 hours.

If you didn't create an account, you can safely ignore this email.

Best regards,
Safatanc Connect Team
            "#,
            username = username,
            verification_url = verification_url
        );

        // Send the email asynchronously
        self.send_email_async(
            email.to_string(),
            subject.to_string(),
            html_content,
            text_content,
        );

        Ok(())
    }

    // Send password reset email
    pub async fn send_password_reset_email(
        &self,
        email: &str,
        username: &str,
        token: &str,
    ) -> Result<(), AppError> {
        // Create reset URL
        let reset_url = format!(
            "{}/auth/reset-password/{}",
            self.email_config.frontend_url, token
        );

        // Email subject
        let subject = "Reset Your Password";

        // Build HTML and text content
        let html_content = format!(
            r#"
            <html>
                <body>
                    <h1>Password Reset</h1>
                    <p>Hello {username},</p>
                    <p>You have requested to reset your password. 
                    Please click the button below to reset your password:</p>
                    <p>
                        <a href="{reset_url}" style="padding: 10px 15px; background-color: #4CAF50; color: white; text-decoration: none; border-radius: 5px;">
                            Reset Password
                        </a>
                    </p>
                    <p>If the button doesn't work, you can also copy and paste the following link into your browser:</p>
                    <p>{reset_url}</p>
                    <p>This link will expire in 24 hours.</p>
                    <p>If you didn't request a password reset, you can safely ignore this email.</p>
                    <p>Best regards,<br/>Safatanc Connect Team</p>
                </body>
            </html>
            "#,
            username = username,
            reset_url = reset_url
        );

        let text_content = format!(
            r#"Password Reset

Hello {username},

You have requested to reset your password. Please use the link below to reset your password:

{reset_url}

This link will expire in 24 hours.

If you didn't request a password reset, you can safely ignore this email.

Best regards,
Safatanc Connect Team
            "#,
            username = username,
            reset_url = reset_url
        );

        // Send the email asynchronously
        self.send_email_async(
            email.to_string(),
            subject.to_string(),
            html_content,
            text_content,
        );

        Ok(())
    }

    // Send email asynchronously in a separate task
    fn send_email_async(
        &self,
        to_email: String,
        subject: String,
        html_content: String,
        text_content: String,
    ) {
        // Clone necessary data for the task
        let email_config = self.email_config.clone();

        // Spawn a new task to send the email
        task::spawn(async move {
            // Create transport inside the task
            let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&email_config.smtp_host)
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to create SMTP transport: {}", e);
                    panic!("Failed to create SMTP transport: {}", e);
                })
                .port(email_config.smtp_port)
                .credentials(Credentials::new(
                    email_config.smtp_username,
                    email_config.smtp_password,
                ))
                .build();

            // Build email message
            let email_result = Message::builder()
                .from(
                    format!(
                        "{} <{}>",
                        email_config.sender_name, email_config.sender_email
                    )
                    .parse()
                    .unwrap_or_else(|e| {
                        tracing::error!("Invalid sender email: {}", e);
                        panic!("Invalid sender email: {}", e);
                    }),
                )
                .to(to_email.parse().unwrap_or_else(|e| {
                    tracing::error!("Invalid recipient email: {}", e);
                    panic!("Invalid recipient email: {}", e);
                }))
                .subject(subject)
                .multipart(
                    MultiPart::alternative()
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_PLAIN)
                                .body(text_content),
                        )
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_HTML)
                                .body(html_content),
                        ),
                );

            match email_result {
                Ok(email) => {
                    // Send email
                    if let Err(e) = transport.send(email).await {
                        tracing::error!("Failed to send email: {}", e);
                    } else {
                        tracing::info!("Email sent successfully");
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to build email: {}", e);
                }
            }
        });
    }

    // Generic send email function (kept for backward compatibility)
    async fn send_email(
        &self,
        to_email: &str,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), AppError> {
        // Create email transport
        let transport = self.create_transport()?;

        // Build email message
        let email = Message::builder()
            .from(
                format!(
                    "{} <{}>",
                    self.email_config.sender_name, self.email_config.sender_email
                )
                .parse()
                .map_err(|e| AppError::Internal(format!("Invalid sender email: {}", e)))?,
            )
            .to(to_email
                .parse()
                .map_err(|e| AppError::Internal(format!("Invalid recipient email: {}", e)))?)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(text_content.to_string()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html_content.to_string()),
                    ),
            )
            .map_err(|e| AppError::Internal(format!("Failed to build email: {}", e)))?;

        // Send email
        transport
            .send(email)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to send email: {}", e)))?;

        Ok(())
    }

    // Generate verification token
    async fn generate_verification_token(&self, user_id: Uuid) -> Result<String, AppError> {
        // Generate a random token
        let token_string = self.generate_random_token(32)?;

        // Create a verification token
        let token_dto = CreateVerificationTokenDto {
            user_id: Some(user_id),
            token_type: TOKEN_TYPE_EMAIL_VERIFICATION.to_string(),
            expires_in: 24 * 60 * 60, // 24 hours in seconds
        };

        // Create token in database
        let token = self
            .token_repo
            .create(&token_dto, &token_string)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(token.token)
    }

    // Generate random token
    fn generate_random_token(&self, length: usize) -> Result<String, AppError> {
        use rand::{distributions::Alphanumeric, Rng};

        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect();

        Ok(token)
    }
}
