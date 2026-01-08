-- Add migration script here
ALTER TABLE garage_shop ADD COLUMN transport_cost INTEGER DEFAULT 0 NOT NULL;
