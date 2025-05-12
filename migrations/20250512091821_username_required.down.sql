-- Revert username column to be nullable
ALTER TABLE users ALTER COLUMN username DROP NOT NULL;