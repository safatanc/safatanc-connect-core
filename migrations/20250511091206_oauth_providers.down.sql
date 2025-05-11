-- Add down migration script here
DROP INDEX IF EXISTS idx_oauth_providers_provider_name;
DROP TABLE IF EXISTS oauth_providers;
