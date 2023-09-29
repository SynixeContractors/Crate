-- No longer used
DROP TABLE reset_kit;

-- Add reason
ALTER TABLE gear_locker_log ADD COLUMN reason VARCHAR(255) NOT NULL DEFAULT '';
ALTER TABLE gear_bank_purchases ADD COLUMN reason VARCHAR(255) NOT NULL DEFAULT '';

-- Update first kit function
CREATE OR REPLACE FUNCTION add_first_kit_to_locker() RETURNS TRIGGER AS $$
DECLARE
    i record;
    trials INTEGER;
    kit JSONB;
BEGIN
    SELECT COUNT(*) INTO trials FROM certifications_trials WHERE trainee = NEW.trainee AND certification = NEW.certification AND passed = TRUE;
    IF trials = 1 THEN
        kit = (SELECT first_kit FROM certifications WHERE id = NEW.certification);
        IF kit IS NOT NULL THEN
            FOR i IN SELECT * FROM jsonb_each(kit) LOOP
                INSERT INTO gear_bank_purchases (member, class, quantity, cost, global, reason)
                    VALUES (NEW.trainee, i.Key, i.Value::INTEGER, (SELECT cost FROM gear_item_current_cost(i.Key)), TRUE, 'first kit');
            END LOOP;
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Copy purchase reason to locker log

CREATE OR REPLACE FUNCTION gear_add_purchase_to_locker()
    RETURNS trigger AS
$func$
BEGIN
    INSERT INTO gear_locker_log (member, class, quantity, created, reason)
    VALUES (NEW.member, NEW.class, NEW.quantity, NEW.created, NEW.reason);
    RETURN null;
END
$func$ LANGUAGE plpgsql;
