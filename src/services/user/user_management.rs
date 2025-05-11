use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::db::error::{DatabaseError, DatabaseResult};
use crate::db::repositories::UserRepository;
use crate::errors::AppError;
use crate::models::user::{CreateUserDto, UpdateUserDto, User, UserResponse};

pub struct UserManagementService {
    user_repo: Arc<UserRepository>,
}

impl UserManagementService {
    pub fn new(user_repo: Arc<UserRepository>) -> Self {
        Self { user_repo }
    }

    // Registrasi pengguna baru
    pub async fn register_user(&self, dto: CreateUserDto) -> Result<User, AppError> {
        // Hash password menggunakan Argon2
        let password_hash = self.hash_password(&dto.password)?;

        // Simpan user ke database
        let user = self
            .user_repo
            .create(&dto, password_hash)
            .await
            .map_err(AppError::Database)?;

        Ok(user)
    }

    // Dapatkan data pengguna berdasarkan ID
    pub async fn get_user_by_id(&self, id: Uuid) -> Result<UserResponse, AppError> {
        let user = self.user_repo.find_by_id(id).await.map_err(|e| match e {
            DatabaseError::NotFound => AppError::NotFound("User tidak ditemukan".into()),
            _ => AppError::Database(e),
        })?;

        Ok(UserResponse::from(user))
    }

    // Dapatkan semua pengguna dengan pagination
    pub async fn get_all_users(
        &self,
        page: u32,
        limit: u32,
    ) -> Result<(Vec<UserResponse>, u64), AppError> {
        let limit = limit as i64;
        let offset = (page as i64 - 1) * limit;

        // Dapatkan data pengguna
        let users = self
            .user_repo
            .find_all(limit, offset)
            .await
            .map_err(AppError::Database)?;

        // Dapatkan total jumlah pengguna untuk pagination
        let total = self.user_repo.count().await.map_err(AppError::Database)? as u64;

        // Konversi ke UserResponse
        let user_responses = users.into_iter().map(UserResponse::from).collect();

        Ok((user_responses, total))
    }

    // Update profil pengguna
    pub async fn update_user(
        &self,
        id: Uuid,
        dto: UpdateUserDto,
    ) -> Result<UserResponse, AppError> {
        let user = self.user_repo.update(id, &dto).await.map_err(|e| match e {
            DatabaseError::NotFound => AppError::NotFound("User tidak ditemukan".into()),
            DatabaseError::Duplicate(msg) => AppError::Validation(msg),
            _ => AppError::Database(e),
        })?;

        Ok(UserResponse::from(user))
    }

    // Update password pengguna
    pub async fn update_password(
        &self,
        id: Uuid,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        // Dapatkan data pengguna
        let user = self.user_repo.find_by_id(id).await.map_err(|e| match e {
            DatabaseError::NotFound => AppError::NotFound("User tidak ditemukan".into()),
            _ => AppError::Database(e),
        })?;

        // Verifikasi password saat ini
        self.verify_password(current_password, &user.password_hash)?;

        // Hash password baru
        let new_password_hash = self.hash_password(new_password)?;

        // Update password di database
        self.user_repo
            .update_password(id, &new_password_hash)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }

    // Hapus pengguna (soft delete)
    pub async fn delete_user(&self, id: Uuid) -> Result<(), AppError> {
        self.user_repo.delete(id).await.map_err(|e| match e {
            DatabaseError::NotFound => AppError::NotFound("User tidak ditemukan".into()),
            _ => AppError::Database(e),
        })?;

        Ok(())
    }

    // Verifikasi email pengguna
    pub async fn verify_email(&self, id: Uuid) -> Result<UserResponse, AppError> {
        let user = self
            .user_repo
            .update_email_verification(id, true)
            .await
            .map_err(|e| match e {
                DatabaseError::NotFound => AppError::NotFound("User tidak ditemukan".into()),
                _ => AppError::Database(e),
            })?;

        Ok(UserResponse::from(user))
    }

    // Helper function untuk hash password
    pub fn hash_password(&self, password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| AppError::Internal(format!("Gagal hash password: {}", e)))
    }

    // Helper function untuk verifikasi password
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<(), AppError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::Internal(format!("Format hash password invalid: {}", e)))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Authentication("Email atau password salah".into()))
    }
}
