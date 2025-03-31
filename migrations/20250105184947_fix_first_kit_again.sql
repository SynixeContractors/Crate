-- Add migration script here
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
                INSERT INTO gear_bank_purchases (member, class, quantity, personal, company, reason)
                    VALUES (NEW.trainee, i.Key, i.Value::INTEGER, 0, (SELECT company_current + personal_current FROM gear_item_current_cost(i.Key)), 'first kit');
            END LOOP;
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
