-- Add migration script here
CREATE TABLE servers (
    id VARCHAR(128) NOT NULL,
    name VARCHAR(128) NOT NULL,
    host VARCHAR(128) NOT NULL,
    port INTEGER NOT NULL,
    password VARCHAR(128) NOT NULL,
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id)
);

CREATE TABLE server_log (
    server VARCHAR(128) NOT NULL,
    member VARCHAR(128) NOT NULL,
    action VARCHAR(32) NOT NULL,
    data JSONB,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (server, member, action, created),
    CONSTRAINT missions_log_server_fk FOREIGN KEY (server) REFERENCES servers (id) ON DELETE CASCADE
);
