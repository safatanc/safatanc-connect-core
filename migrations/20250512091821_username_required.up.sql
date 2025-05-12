-- Make username column required (NOT NULL)
-- First fill any NULL usernames with a default value
UPDATE users SET username = 'user_' || substring(id::text from 1 for 8) WHERE username IS NULL;

-- Then make the column NOT NULL
ALTER TABLE users ALTER COLUMN username SET NOT NULL;