-- Add migration script here
ALTER TABLE missions ADD COLUMN archived BOOLEAN NOT NULL DEFAULT TRUE;
