-- Add migration script here
CREATE TABLE github_usernames (
    member VARCHAR(128) NOT NULL,
    github TEXT NOT NULL,
    PRIMARY KEY (member)
);
