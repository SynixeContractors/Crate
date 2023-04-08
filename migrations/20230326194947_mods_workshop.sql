-- Add migration script here
CREATE TABLE Mods_Workshop (
    workshop_id VARCHAR(128) NOT NULL,
    name VARCHAR(255) NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (workshop_id)
);
