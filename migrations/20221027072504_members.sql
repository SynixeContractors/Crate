-- Add migration script here
CREATE TABLE IF NOT EXISTS members_steam (
    user_id VARCHAR(128) PRIMARY KEY,
    steam_id VARCHAR(128) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS members_dlc (
    user_id VARCHAR(128) PRIMARY KEY,
    dlc_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
