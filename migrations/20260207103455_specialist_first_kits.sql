-- Add migration script here
ALTER TABLE certifications_first_kit ADD COLUMN specialist BOOLEAN NOT NULL DEFAULT TRUE;
