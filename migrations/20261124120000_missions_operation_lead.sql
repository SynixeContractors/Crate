-- The member leading a scheduled mission, claimed from the 24 hour leadership reminder
ALTER TABLE missions_schedule ADD COLUMN IF NOT EXISTS operation_lead VARCHAR(128);
