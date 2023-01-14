-- Add migration script here
CREATE TABLE missions_maps (
    map VARCHAR(64) PRIMARY KEY,
    archived BOOLEAN NOT NULL DEFAULT FALSE
);
