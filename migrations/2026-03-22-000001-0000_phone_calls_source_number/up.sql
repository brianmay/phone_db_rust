-- Add source_number as nullable first so we can back-fill from the contact's phone_number.
ALTER TABLE phone_calls ADD COLUMN source_number VARCHAR(255);

UPDATE phone_calls
SET source_number = contacts.phone_number
FROM contacts
WHERE phone_calls.contact_id = contacts.id;

-- Now enforce NOT NULL.
ALTER TABLE phone_calls ALTER COLUMN source_number SET NOT NULL;
