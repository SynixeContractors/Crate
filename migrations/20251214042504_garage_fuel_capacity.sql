-- Add migration script here
ALTER TABLE garage_shop ADD COLUMN fuel_capacity INT DEFAULT 0 NOT NULL;
