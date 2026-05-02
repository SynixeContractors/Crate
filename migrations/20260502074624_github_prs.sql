-- Add migration script here
CREATE TABLE github_prs (
    id INTEGER PRIMARY KEY,
    thread TEXT NOT NULL
);
