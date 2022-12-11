-- Add migration script here
ALTER TABLE gear_items ADD COLUMN enabled BOOLEAN NOT NULL DEFAULT TRUE;
