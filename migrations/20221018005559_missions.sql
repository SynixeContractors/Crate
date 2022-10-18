CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

DO $$ BEGIN
    CREATE TYPE mission_type AS ENUM ('contract', 'subcontract', 'training', 'special', 'other');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

CREATE TABLE IF NOT EXISTS missions (
    id VARCHAR(256) NOT NULL,
    name VARCHAR(128) NOT NULL,
    summary VARCHAR(512) NOT NULL,
    description TEXT NOT NULL,
    type mission_type NOT NULL,
    PRIMARY KEY (id)
);
CREATE INDEX IF NOT EXISTS missions_type_idx ON missions (type);

CREATE TABLE IF NOT EXISTS missions_schedule (
    id UUID NOT NULL DEFAULT uuid_generate_v4(),
    mission VARCHAR(128) NOT NULL,
    schedule_message_id VARCHAR(64) DEFAULT NULL,
    start_at TIMESTAMP NOT NULL,
    PRIMARY KEY (id),
    CONSTRAINT missions_schedule_mission_fk FOREIGN KEY (mission) REFERENCES missions (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS missions_schedule_players (
    schedule_id UUID NOT NULL,
    player VARCHAR(128) NOT NULL,
    joined boolean NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (schedule_id, player),
    CONSTRAINT missions_schedule_players_schedule_id_fk FOREIGN KEY (schedule_id) REFERENCES missions_schedule (id) ON DELETE CASCADE
);
