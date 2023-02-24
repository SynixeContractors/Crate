-- Add migration script here
CREATE TABLE reset_kit (
    member VARCHAR(128) NOT NULL,
    cert UUID NOT NULL,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (member, cert)
);
