-- Add migration script here
CREATE TABLE IF NOT EXISTS recruitment_seen (
    link VARCHAR(255) NOT NULL,
    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (link)
);
