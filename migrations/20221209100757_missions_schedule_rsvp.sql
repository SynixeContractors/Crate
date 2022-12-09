-- Add migration script here
DO $$ BEGIN
    CREATE TYPE missions_schedule_rsvp_state AS ENUM ('yes', 'maybe', 'no');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

CREATE TABLE IF NOT EXISTS missions_schedule_rsvp (
    mission VARCHAR(128) NOT NULL,
    member VARCHAR(128) NOT NULL,
    state missions_schedule_rsvp_state NOT NULL,
    details TEXT,
    PRIMARY KEY (mission, member)
);
