CREATE TABLE campaigns_loadouts (
    campaign UUID NOT NULL,
    member VARCHAR(128),
    loadout TEXT NOT NULL,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (campaign, member)
);

CREATE OR REPLACE FUNCTION campaigns_loadouts_update_created_on_update() RETURNS TRIGGER AS $$
BEGIN
    NEW.created = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER campaigns_loadouts_update_created_on_update
BEFORE UPDATE ON campaigns_loadouts
FOR EACH ROW
EXECUTE PROCEDURE campaigns_loadouts_update_created_on_update();

CREATE TABLE campaigns_loadouts_log (
    campaign UUID NOT NULL,
    member VARCHAR(128) NOT NULL,
    loadout TEXT NOT NULL,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (campaign, member, created)
);

CREATE INDEX campaigns_old_loadouts_campaign_member_idx ON campaigns_loadouts_log (campaign, member);

-- Move a row to campaigns_loadouts_log when updated in loadouts
CREATE OR REPLACE FUNCTION campaigns_move_to_old_loadouts()
    RETURNS trigger AS
$func$
BEGIN
    INSERT INTO campaigns_loadouts_log (campaign, member, loadout, created)
    VALUES (OLD.campaign, OLD.member, OLD.loadout, OLD.created);
    
    DELETE FROM campaigns_loadouts_log 
    WHERE campaign = NEW.campaign 
        AND member = NEW.member 
        AND created < (
            SELECT created 
            FROM campaigns_loadouts_log 
            WHERE campaign = NEW.campaign 
                AND member = NEW.member 
            ORDER BY created DESC 
            LIMIT 1 OFFSET 1000
        );
    RETURN NEW;
END
$func$ LANGUAGE plpgsql;

CREATE TRIGGER campaigns_move_to_old_loadouts
BEFORE UPDATE ON campaigns_loadouts
FOR EACH ROW
EXECUTE PROCEDURE campaigns_move_to_old_loadouts();
