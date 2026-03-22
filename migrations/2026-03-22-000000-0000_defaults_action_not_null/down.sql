-- Revert defaults.action to nullable
ALTER TABLE defaults ALTER COLUMN action DROP NOT NULL;
