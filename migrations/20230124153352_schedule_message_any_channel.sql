-- Change schedule_message_id to VARCHAR(255) and prepend `700888805137318039:` to all existing non-null values
ALTER TABLE missions_schedule ALTER COLUMN schedule_message_id TYPE VARCHAR(255);
UPDATE missions_schedule SET schedule_message_id = CONCAT('700888805137318039:', schedule_message_id) WHERE schedule_message_id IS NOT NULL;
