-- Add migration script here
ALTER TABLE reset_kit ADD COLUMN specialist BOOLEAN NOT NULL DEFAULT TRUE;
