-- Add migration script here
CREATE OR REPLACE FUNCTION garage_purchases_insert() RETURNS TRIGGER AS $$
DECLARE purchasedBase UUID;
DECLARE state JSONB;
BEGIN
    SELECT base INTO purchasedBase FROM garage_shop WHERE id = NEW.id;
    IF purchasedBase IS NULL THEN
        SELECT json_build_object(
            'tex_source', (SELECT texture_source FROM garage_colors where id = NEW.id AND name = NEW.color LIMIT 1)
        ) INTO state;
        INSERT INTO garage_vehicles (plate, id, stored, state) VALUES (NEW.plate, NEW.id, TRUE, state);
    ELSE
        INSERT INTO garage_addons (id, count) VALUES (NEW.id, 1) ON CONFLICT (id) DO UPDATE SET count = garage_addons.count + 1;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
