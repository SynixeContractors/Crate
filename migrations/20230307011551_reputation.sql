-- Add migration script here
CREATE TABLE reputation_events (
    id UUID NOT NULL DEFAULT uuid_generate_v4(),
    member VARCHAR(128) NOT NULL,
    event VARCHAR(128) NOT NULL,
    reputation NUMERIC NOT NULL,
    data JSONB NOT NULL,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id)
);

CREATE INDEX reputation_events_member_idx ON reputation_events (member);
CREATE INDEX reputation_events_event_idx ON reputation_events (event);
CREATE INDEX reputation_events_created_idx ON reputation_events (created);

-- A function that calculates the current reputation of the community
-- Recent events are weighted more heavily than older events
CREATE OR REPLACE FUNCTION reputation() RETURNS TABLE (reputation NUMERIC) AS $$
DECLARE
    -- The number of days to consider when calculating reputation
    days NUMERIC := 60;
    -- The number of seconds in a day
    seconds_in_day NUMERIC := 86400;
    -- The number of seconds in the number of days we're considering
    seconds_in_days NUMERIC := days * seconds_in_day;
    -- The current time
    now TIMESTAMPTZ := NOW();
    -- The time in the past that we're considering
    past TIMESTAMPTZ := now - (days || ' days')::INTERVAL;
    -- The total reputation of the community
    total NUMERIC := 0;
    -- The number of events that have been considered
    count NUMERIC := 0;
    -- A row from the reputation_events table
    row RECORD;
BEGIN
    FOR row IN SELECT * FROM reputation_events WHERE created >= past LOOP
        -- Calculate the weight of the event
        -- The weight is the number of seconds since the event divided by the
        -- number of seconds in the number of days we're considering
        -- This means that events that happened recently are weighted more
        -- heavily than events that happened a long time ago
        -- The weight is then multiplied by the reputation value of the event
        -- This means that events that have a higher reputation value are
        -- weighted more heavily than events that have a lower reputation value
        -- The weight is then added to the total reputation of the community
        total := total + EXTRACT(EPOCH FROM now - row.created) / seconds_in_days * row.reputation;
        -- Increment the number of events that have been considered
        count := count + 1;
    END LOOP;
    -- Return the average reputation of the community
    RETURN QUERY SELECT total / count;
END;
$$ LANGUAGE plpgsql;
