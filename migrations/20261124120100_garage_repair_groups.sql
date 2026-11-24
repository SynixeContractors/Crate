-- Parts of a vehicle that are repaired together with /garage repair, e.g. "Engine" or "Wheels"
CREATE TABLE IF NOT EXISTS garage_repair_groups (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    -- Shown to players
    name VARCHAR(64) NOT NULL UNIQUE,
    -- Hitpoint names in the group, as case-insensitive LIKE patterns, e.g. {'HitEngine%', 'HitFuel'}.
    -- A hitpoint should only be in one group, or repairing it is charged for each.
    hitpoints TEXT[] NOT NULL,
    -- Cost to repair one hitpoint in the group from fully damaged, scaled by how damaged it is
    cost INTEGER NOT NULL CHECK (cost >= 0)
);
