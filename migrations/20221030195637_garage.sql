-- Add migration script here
CREATE TABLE IF NOT EXISTS garage_shop (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    base UUID,
    class VARCHAR(255) NOT NULL,
    name VARCHAR(128) NOT NULL,
    cost INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS garage_shop_base_idx ON garage_shop (base);

CREATE TABLE IF NOT EXISTS garage_purchases (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member VARCHAR(128) NOT NULL,
    cost INTEGER NOT NULL,
    created TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION garage_purchases_insert() RETURNS TRIGGER AS $$
DECLARE purchasedBase UUID;
BEGIN
    SELECT base INTO purchasedBase FROM garage_shop WHERE id = NEW.id;
    IF purchasedBase IS NULL THEN
        INSERT INTO garage_vehicles (plate, id, stored, state) VALUES ('', NEW.id, TRUE, '{}');
    ELSE
        INSERT INTO garage_addons (id, count) VALUES (NEW.id, 1) ON CONFLICT (id) DO UPDATE SET count = garage_addons.count + 1;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER garage_purchases_insert AFTER INSERT ON garage_purchases FOR EACH ROW EXECUTE PROCEDURE garage_purchases_insert();

CREATE TABLE IF NOT EXISTS garage_vehicles (
    plate VARCHAR(10) PRIMARY KEY,
    id UUID NOT NULL,
    addon UUID,
    stored BOOLEAN NOT NULL,
    state JSONB,
    updated TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT garage_vehicles_id FOREIGN KEY (id) REFERENCES garage_shop (id) ON DELETE CASCADE,
    CONSTRAINT garage_vehicles_state CHECK (jsonb_typeof(state) = 'object')
);
CREATE INDEX IF NOT EXISTS garage_vehicles_id_idx ON garage_vehicles (id);
CREATE INDEX IF NOT EXISTS garage_vehicles_stored_idx ON garage_vehicles (stored);

CREATE TABLE IF NOT EXISTS garage_addons (
    id UUID PRIMARY KEY,
    count INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS garage_log (
    plate VARCHAR(10) NOT NULL,
    member VARCHAR(128) NOT NULL,
    action VARCHAR(16) NOT NULL,
    data JSONB,
    created TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT garage_log_plate FOREIGN KEY (plate) REFERENCES garage_vehicles (plate) ON DELETE CASCADE,
    CONSTRAINT garage_log_action CHECK (action IN ('store', 'retrieve', 'attach', 'detach', 'state')),
    CONSTRAINT garage_log_data CHECK (
        (action IN ('store', 'state') AND jsonb_typeof(data) = 'object') OR
        (action IN ('attach', 'detach') AND jsonb_typeof(data) = 'object') OR
        (action IN ('retrieve') AND data IS NULL)
    )
);

CREATE OR REPLACE FUNCTION garage_log_has_addon() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.action IN ('attach', 'detach') THEN
        IF NEW.data->>'addon' IS NULL THEN
            RAISE EXCEPTION 'addon is required';
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER garage_log_has_addon
    BEFORE INSERT OR UPDATE ON garage_log
    FOR EACH ROW
    EXECUTE PROCEDURE garage_log_has_addon();

CREATE OR REPLACE FUNCTION garage_log_determine_state(VARCHAR(10)) RETURNS void AS $$
DECLARE lastAction VARCHAR(16);
DECLARE lastData JSONB;
BEGIN
    SELECT action, data INTO lastAction, lastData FROM garage_log WHERE plate = $1 ORDER BY created DESC LIMIT 1;
    IF lastAction = 'store' THEN
        UPDATE garage_vehicles SET stored = TRUE, state = lastData WHERE plate = $1;
    ELSIF lastAction = 'retrieve' THEN
        UPDATE garage_vehicles SET stored = FALSE WHERE plate = $1;
    ELSIF lastAction = 'state' THEN
        UPDATE garage_vehicles SET state = lastData WHERE plate = $1;
    ELSEIF lastAction = 'attach' THEN
        UPDATE garage_vehicles SET addon = (lastData->>'addon')::uuid WHERE plate = $1;
        UPDATE garage_addons SET count = count - 1 WHERE id = (lastData->>'addon')::uuid;
    ELSEIF lastAction = 'detach' THEN
        UPDATE garage_vehicles SET addon = (lastData->>'addon')::uuid WHERE plate = $1;
        UPDATE garage_addons SET count = count + 1 WHERE id = (lastData->>'addon')::uuid;
    END IF;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION garage_log_determine_state_insert_update() RETURNS TRIGGER AS $$
BEGIN
    PERFORM garage_log_determine_state(NEW.plate);
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER garage_log_determine_state_insert_update
    AFTER INSERT OR UPDATE ON garage_log
    FOR EACH ROW
    EXECUTE PROCEDURE garage_log_determine_state_insert_update();

CREATE OR REPLACE FUNCTION garage_log_determine_state_delete() RETURNS TRIGGER AS $$
BEGIN
    PERFORM garage_log_determine_state(OLD.plate);
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER garage_log_determine_state_delete
    AFTER DELETE ON garage_log
    FOR EACH ROW
    EXECUTE PROCEDURE garage_log_determine_state_delete();
