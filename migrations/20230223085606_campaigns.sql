-- Add migration script here
CREATE TABLE campaigns (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    map VARCHAR(64) NOT NULL,
    name VARCHAR(128) NOT NULL
);

CREATE TABLE campaigns_objects (
    id UUID DEFAULT uuid_generate_v4(),
    campaign UUID NOT NULL,
    class VARCHAR NOT NULL,
    data JSONB NOT NULL,
    PRIMARY KEY (campaign, id),
    FOREIGN KEY (campaign) REFERENCES campaigns(id) ON DELETE CASCADE
);

CREATE TABLE campaigns_groups (
    id UUID DEFAULT uuid_generate_v4(),
    campaign UUID NOT NULL,
    data JSONB NOT NULL,
    PRIMARY KEY (campaign, id),
    FOREIGN KEY (campaign) REFERENCES campaigns(id) ON DELETE CASCADE
);

CREATE TABLE campaigns_units (
    id UUID DEFAULT uuid_generate_v4(),
    campaign UUID NOT NULL,
    class VARCHAR NOT NULL,
    "group" UUID NOT NULL,
    data JSONB NOT NULL,
    PRIMARY KEY (campaign, id),
    FOREIGN KEY (campaign) REFERENCES campaigns(id) ON DELETE CASCADE
);

CREATE TABLE campaigns_markers (
    name VARCHAR NOT NULL,
    campaign UUID NOT NULL,
    data JSONB NOT NULL,
    PRIMARY KEY (campaign, name),
    FOREIGN KEY (campaign) REFERENCES campaigns(id) ON DELETE CASCADE
);
