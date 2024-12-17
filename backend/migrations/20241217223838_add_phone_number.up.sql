ALTER TABLE phone_calls
ADD COLUMN phone_number character varying(255);
UPDATE phone_calls
SET phone_number = contacts.phone_number
FROM contacts
WHERE phone_calls.contact_id = contacts.id;
ALTER TABLE phone_calls
ALTER COLUMN phone_number
SET NOT NULL;