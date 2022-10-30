-- Add migration script here
CREATE TABLE IF NOT EXISTS members_steam (
    member VARCHAR(128) PRIMARY KEY,
    steam_id VARCHAR(128) NOT NULL,
    created TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS members_dlc (
    member VARCHAR(128) PRIMARY KEY,
    dlc_id INTEGER NOT NULL,
    created TIMESTAMP NOT NULL DEFAULT NOW()
);
