-- Add migration script here
ALTER TABLE recruitment_seen ALTER created TYPE TIMESTAMPTZ USING created AT TIME ZONE 'UTC';
ALTER TABLE recruitment_seen ALTER created SET DEFAULT NOW();
ALTER TABLE recruitment_replied ALTER created TYPE TIMESTAMPTZ USING created AT TIME ZONE 'UTC';
ALTER TABLE recruitment_replied ALTER created SET DEFAULT NOW();

ALTER TABLE missions_schedule ALTER start TYPE TIMESTAMPTZ USING start AT TIME ZONE 'UTC';
ALTER TABLE missions_schedule ALTER start SET DEFAULT NOW();
ALTER TABLE missions_schedule_players ALTER created TYPE TIMESTAMPTZ USING created AT TIME ZONE 'UTC';
ALTER TABLE missions_schedule_players ALTER created SET DEFAULT NOW();
