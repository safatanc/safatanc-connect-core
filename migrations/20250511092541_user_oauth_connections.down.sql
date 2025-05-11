-- Add down migration script here
DROP INDEX IF EXISTS idx_user_oauth_connections_provider_id;
DROP INDEX IF EXISTS idx_user_oauth_connections_user_id;
DROP TABLE IF EXISTS user_oauth_connections; 