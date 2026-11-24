-- Reachout messages staff sent to members who went inactive, so they aren't sent again too soon
CREATE TABLE IF NOT EXISTS members_reachouts (
    member VARCHAR(128) NOT NULL,
    -- The staff member who sent it
    sent_by VARCHAR(128) NOT NULL,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (member, created)
);
