use crate::errors::AppError;
use regex::Regex;
use validator::{Validate, ValidationError};

// Validate email format
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    let email_regex = Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap();

    if !email_regex.is_match(email) {
        return Err(ValidationError::new("invalid_email_format"));
    }

    Ok(())
}

// Validate password strength
pub fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    // Check for minimum length (8 characters)
    if password.len() < 8 {
        return Err(ValidationError::new("password_too_short"));
    }

    // Check for uppercase letters
    let uppercase_regex = Regex::new(r"[A-Z]").unwrap();
    if !uppercase_regex.is_match(password) {
        return Err(ValidationError::new("password_no_uppercase"));
    }

    // Check for numbers
    let number_regex = Regex::new(r"[0-9]").unwrap();
    if !number_regex.is_match(password) {
        return Err(ValidationError::new("password_no_number"));
    }

    // Check for special characters
    let special_char_regex = Regex::new(r"[!@#$%^&*(),.?:{}|<>]").unwrap();
    if !special_char_regex.is_match(password) {
        return Err(ValidationError::new("password_no_special_char"));
    }

    Ok(())
}

// Validate username format (alphanumeric, underscore, hyphen, minimum 3 characters)
pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    let username_regex = Regex::new(r"^[a-zA-Z0-9_-]{3,30}$").unwrap();

    if !username_regex.is_match(username) {
        return Err(ValidationError::new("invalid_username_format"));
    }

    Ok(())
}

// Helper function to convert validation errors to AppError
pub fn validation_err_to_app_error(error: validator::ValidationErrors) -> AppError {
    let mut error_messages = String::new();

    for (field, errors) in error.field_errors() {
        for error in errors {
            let message = match error.code.as_ref() {
                "password_too_short" => "Password must be at least 8 characters",
                "password_no_uppercase" => "Password must contain at least one uppercase letter",
                "password_no_number" => "Password must contain at least one number",
                "password_no_special_char" => "Password must contain at least one special character",
                "invalid_email_format" => "Invalid email format",
                "invalid_username_format" => "Username must be 3-30 characters and contain only letters, numbers, underscores, or hyphens",
                _ => error.message.as_ref().map_or(
                    error.code.as_ref(), |m| m.as_ref()
                ),
            };

            if !error_messages.is_empty() {
                error_messages.push_str("; ");
            }
            error_messages.push_str(&format!("{}: {}", field, message));
        }
    }

    if error_messages.is_empty() {
        error_messages = "Validation failed".to_string();
    }

    AppError::Validation(error_messages)
}
