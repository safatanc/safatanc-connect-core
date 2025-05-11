-- Add down migration script here
DROP INDEX IF EXISTS idx_verification_tokens_type;
DROP INDEX IF EXISTS idx_verification_tokens_token;
DROP INDEX IF EXISTS idx_verification_tokens_user_id;
DROP TABLE IF EXISTS verification_tokens; 