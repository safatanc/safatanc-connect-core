use uuid::Uuid;

use crate::db::error::DatabaseError;
use crate::db::repositories::UserRepository;
use crate::errors::AppError;
use crate::models::user::{AuthResponse, LoginDto, UserResponse};
use crate::services::auth::token::TokenService;
use crate::services::user::user_management::UserManagementService;

pub struct AuthService<'a> {
    user_repo: &'a UserRepository,
    token_service: &'a TokenService,
    user_management: &'a UserManagementService<'a>,
}

impl<'a> AuthService<'a> {
    pub fn new(
        user_repo: &'a UserRepository,
        token_service: &'a TokenService,
        user_management: &'a UserManagementService<'a>,
    ) -> Self {
        Self {
            user_repo,
            token_service,
            user_management,
        }
    }

    // Login pengguna
    pub async fn login(&self, dto: LoginDto) -> Result<AuthResponse, AppError> {
        // Cari user berdasarkan email
        let user = self
            .user_repo
            .find_by_email(&dto.email)
            .await
            .map_err(|e| match e {
                DatabaseError::NotFound => {
                    AppError::Authentication("Email atau password salah".into())
                }
                _ => AppError::Database(e),
            })?;

        // Verifikasi password
        self.user_management
            .verify_password(&dto.password, &user.password_hash)?;

        // Update waktu login terakhir
        let updated_user = self
            .user_repo
            .update_last_login(user.id)
            .await
            .map_err(AppError::Database)?;

        // Generate token JWT
        let (token, refresh_token) = self.token_service.generate_tokens(&updated_user)?;

        // Buat response
        let user_response = UserResponse::from(updated_user);
        Ok(AuthResponse {
            user: user_response,
            token,
            refresh_token,
        })
    }

    // Refresh token untuk mendapatkan token baru
    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<String, AppError> {
        // Verifikasi refresh token
        let claims = self.token_service.verify_token(refresh_token)?;

        // Pastikan user masih ada dan aktif
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Authentication("Token tidak valid".into()))?;

        let _user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(|e| match e {
                DatabaseError::NotFound => AppError::Authentication("User tidak ditemukan".into()),
                _ => AppError::Database(e),
            })?;

        // Generate token baru
        let new_token = self.token_service.refresh_token(refresh_token)?;

        Ok(new_token)
    }

    // Logout pengguna (dapat dikembangkan untuk penanganan token blacklist, dll)
    pub async fn logout(&self, user_id: Uuid) -> Result<(), AppError> {
        // Untuk implementasi sederhana, cukup verifikasi bahwa user ada
        let _user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(|e| match e {
                DatabaseError::NotFound => AppError::NotFound("User tidak ditemukan".into()),
                _ => AppError::Database(e),
            })?;

        // Di sini bisa menambahkan logika seperti:
        // - Menambahkan refresh token ke blacklist
        // - Menghapus sesi pengguna dari database
        // - Logging aktivitas logout, dll

        Ok(())
    }

    // Verifikasi token dan ambil user ID
    pub fn validate_token(&self, token: &str) -> Result<Uuid, AppError> {
        self.token_service.get_user_id_from_token(token)
    }
}
