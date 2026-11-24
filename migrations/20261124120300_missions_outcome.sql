-- How a scheduled mission went, from its AAR, recorded when it's paid
ALTER TABLE missions_schedule ADD COLUMN IF NOT EXISTS outcome VARCHAR(16)
    CHECK (outcome IN ('success', 'partial', 'failure'));
