-- Make defaults.action NOT NULL, defaulting any existing NULLs to 'allow'
ALTER TABLE defaults ALTER COLUMN action SET DEFAULT 'allow';
UPDATE defaults SET action = 'allow' WHERE action IS NULL;
ALTER TABLE defaults ALTER COLUMN action SET NOT NULL;
ALTER TABLE defaults ALTER COLUMN action DROP DEFAULT;
