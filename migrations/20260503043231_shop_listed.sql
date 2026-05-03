-- Add migration script here
ALTER TABLE gear_items ADD COLUMN listed BOOLEAN NOT NULL DEFAULT TRUE;
