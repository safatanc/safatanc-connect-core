use crate::db::repositories::UserRepository;
use crate::errors::AppError;
use crate::models::user::{AuthResponse, CreateUserDto, UserResponse};
use crate::services::auth::token::TokenService;
use crate::services::user::user_management::UserManagementService;

pub struct RegistrationService<'a> {
    user_management: &'a UserManagementService<'a>,
    token_service: &'a TokenService,
}

impl<'a> RegistrationService<'a> {
    pub fn new(
        user_management: &'a UserManagementService<'a>,
        token_service: &'a TokenService,
    ) -> Self {
        Self {
            user_management,
            token_service,
        }
    }

    // Registrasi pengguna baru dan generate token
    pub async fn register(&self, dto: CreateUserDto) -> Result<AuthResponse, AppError> {
        // Registrasi user melalui user management service
        let user = self.user_management.register_user(dto).await?;

        // Generate token untuk user baru
        let (token, refresh_token) = self.token_service.generate_tokens(&user)?;

        // Buat response
        let user_response = UserResponse::from(user);
        Ok(AuthResponse {
            user: user_response,
            token,
            refresh_token,
        })
    }

    // Verifikasi email setelah pendaftaran
    pub async fn complete_email_verification(&self, token: &str) -> Result<(), AppError> {
        // Verify token dan dapatkan user ID
        let user_id = self.token_service.get_user_id_from_token(token)?;

        // Update status verifikasi email
        self.user_management.verify_email(user_id).await?;

        Ok(())
    }
}
