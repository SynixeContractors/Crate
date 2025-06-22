-- Create certifications_first_kit
CREATE TABLE certifications_first_kit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    certification UUID NOT NULL REFERENCES certifications(id) ON DELETE CASCADE,
    name VARCHAR(128) NOT NULL,
    description TEXT,
    first_kit JSONB NOT NULL,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Move existing first_kit data to certifications_first_kit
INSERT INTO certifications_first_kit (certification, first_kit, name)
SELECT id, first_kit, name
FROM certifications
WHERE first_kit IS NOT NULL;

-- Remove first_kit from certifications
ALTER TABLE certifications
DROP COLUMN first_kit;
