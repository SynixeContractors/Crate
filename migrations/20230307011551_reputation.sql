-- Add migration script here
CREATE TABLE reputation_events (
    member VARCHAR(128) NOT NULL,
    event VARCHAR(128) NOT NULL,
    reputation INTEGER NOT NULL,
    data JSONB NOT NULL,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (member, event, created)
);

CREATE INDEX reputation_events_member_idx ON reputation_events (member);
CREATE INDEX reputation_events_event_idx ON reputation_events (event);
CREATE INDEX reputation_events_created_idx ON reputation_events (created);

-- A function that calculates the current reputation of the community
-- Recent events are weighted more heavily than older events
CREATE OR REPLACE FUNCTION reputation(at TIMESTAMPTZ) RETURNS DOUBLE PRECISION AS $$
DECLARE
    -- The number of days to consider when calculating reputation
    days NUMERIC := 60;
    -- The time in the past that we're considering
    past TIMESTAMPTZ := at - (days || ' days')::INTERVAL;
    -- The total reputation of the community
    total NUMERIC := 0;
    -- A row from the reputation_events table
    row RECORD;
BEGIN
    FOR row IN SELECT * FROM reputation_events WHERE created >= past AND created <= at LOOP
        -- use weight_i = e^(-lambda * t_i) to calculate the weight of each event
        -- where lambda is the decay rate and t_i is the time since the event
        total := total + row.reputation * EXP(-0.0001 * EXTRACT(EPOCH FROM at - row.created)/3600);
    END LOOP;
    RETURN total;
END;
$$ LANGUAGE plpgsql;
