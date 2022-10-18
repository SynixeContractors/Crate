CREATE TABLE IF NOT EXISTS missions_list (
    id VARCHAR(256) NOT NULL,
    name VARCHAR(128) NOT NULL,
    summary VARCHAR(512) NOT NULL,
    description TEXT NOT NULL,
    type INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id)
);
CREATE INDEX IF NOT EXISTS missions_list_type_idx ON missions_list (type);

CREATE TABLE IF NOT EXISTS missions_schedule (
    id UUID NOT NULL,
    mission VARCHAR(128) NOT NULL,
    start_at TIMESTAMP NOT NULL,
    PRIMARY KEY (id),
    CONSTRAINT missions_schedule_mission_fk FOREIGN KEY (mission) REFERENCES missions_list (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS missions_schedule_players (
    schedule_id UUID NOT NULL,
    player VARCHAR(128) NOT NULL,
    joined boolean NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (schedule_id, player),
    CONSTRAINT missions_schedule_players_schedule_id_fk FOREIGN KEY (schedule_id) REFERENCES missions_schedule (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS missions_schedule_messages (
    schedule_id UUID NOT NULL,
    message_id BIGINT NOT NULL,
    PRIMARY KEY (schedule_id),
    CONSTRAINT missions_schedule_messages_schedule_id_fk FOREIGN KEY (schedule_id) REFERENCES missions_schedule (id) ON DELETE CASCADE
);
