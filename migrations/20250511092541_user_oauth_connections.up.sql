-- Add up migration script here
CREATE TABLE IF NOT EXISTS user_oauth_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    provider_id UUID NOT NULL REFERENCES oauth_providers (id) ON DELETE CASCADE,
    provider_user_id VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    name VARCHAR(255),
    avatar_url VARCHAR(255),
    access_token TEXT,
    refresh_token TEXT,
    expires_at TIMESTAMPTZ,
    raw_user_info JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ,
    UNIQUE (user_id, provider_id),
    UNIQUE (provider_id, provider_user_id)
);

CREATE INDEX IF NOT EXISTS idx_user_oauth_connections_user_id ON user_oauth_connections (user_id);

CREATE INDEX IF NOT EXISTS idx_user_oauth_connections_provider_id ON user_oauth_connections (provider_id);