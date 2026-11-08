-- Advanced Equipment device state, kept separate from campaigns_objects so that
-- removing an object from the mission never removes its saved device state.
CREATE TABLE campaigns_ae3_devices (
    id UUID NOT NULL,
    campaign UUID NOT NULL,
    class VARCHAR NOT NULL,
    state JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (campaign, id),
    FOREIGN KEY (campaign) REFERENCES campaigns(id) ON DELETE CASCADE
);
