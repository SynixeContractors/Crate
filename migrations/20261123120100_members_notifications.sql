-- Direct messages members have opted in to, all off by default
CREATE TABLE IF NOT EXISTS members_notifications (
    member VARCHAR(128) PRIMARY KEY,
    -- Mission payouts
    payday BOOLEAN NOT NULL DEFAULT FALSE,
    -- 15 minutes before a mission they RSVP'd to, if they aren't on the server
    mission_reminder BOOLEAN NOT NULL DEFAULT FALSE
);

-- Finds a player's latest connect or disconnect, for mission reminders. Partial, as chat is most of the log.
CREATE INDEX IF NOT EXISTS servers_log_steam_connection_idx
    ON servers_log (steam, created DESC)
    WHERE action IN ('connected', 'disconnected');
