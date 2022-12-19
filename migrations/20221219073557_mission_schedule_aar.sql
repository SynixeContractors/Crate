-- Add migration script here
ALTER TABLE missions_schedule ADD COLUMN aar_message_id VARCHAR(64) NULL;
