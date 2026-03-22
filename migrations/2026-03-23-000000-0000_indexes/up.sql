-- Composite index on phone_calls(contact_id, inserted_at DESC, id DESC).
-- Serves all three access patterns:
--   1. get_phone_calls_for_contact: WHERE contact_id = $1 ORDER BY inserted_at DESC, id DESC
--   2. COUNT(*) WHERE contact_id = $1  (correlated subquery in contact search)
--   3. get_phone_call_count: COUNT(*) WHERE contact_id = $1
CREATE INDEX idx_phone_calls_contact_id_inserted_at
    ON phone_calls (contact_id, inserted_at DESC, id DESC);

-- Covers the global paginated phone call search sort:
--   search_phone_calls_paginated: ORDER BY inserted_at DESC, id DESC
CREATE INDEX idx_phone_calls_inserted_at
    ON phone_calls (inserted_at DESC, id DESC);

-- Covers the paginated contact search sort and keyset cursor:
--   search_contacts_paginated: ORDER BY name ASC NULLS LAST, id ASC
CREATE INDEX idx_contacts_name_id
    ON contacts (name ASC NULLS LAST, id ASC);

-- Covers expired session cleanup:
--   delete_expired: DELETE FROM session WHERE expiry_date < NOW()
CREATE INDEX idx_session_expiry_date
    ON session (expiry_date);
