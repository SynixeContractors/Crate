-- Add migration script here
ALTER TABLE gear_bank_purchases DROP COLUMN cost;
ALTER TABLE gear_bank_purchases DROP COLUMN global;
ALTER TABLE gear_bank_purchases ADD COLUMN personal INTEGER NOT NULL DEFAULT 0;
ALTER TABLE gear_bank_purchases ADD COLUMN company INTEGER NOT NULL DEFAULT 0;

ALTER TABLE gear_cost RENAME COLUMN cost TO personal;
ALTER TABLE gear_cost ADD COLUMN company INTEGER NOT NULL DEFAULT 0;

-- Retrieve the balance of a member
-- Take into account the deposits, purchases, and transfers
-- Do not include global purchases
CREATE OR REPLACE FUNCTION gear_get_member_balance(VARCHAR(128)) RETURNS integer AS $$
DECLARE balance integer;
BEGIN
    SELECT (
        gear_bank_deposits.sum - gear_bank_purchases.sum - transfers_out.sum + transfers_in.sum
    ) INTO balance
    FROM (
        SELECT COALESCE(SUM(amount), 0) AS sum
        FROM gear_bank_deposits
        WHERE member = $1
    ) AS gear_bank_deposits, (
        SELECT COALESCE(SUM(personal * quantity), 0) AS sum
        FROM gear_bank_purchases
        WHERE member = $1
    ) AS gear_bank_purchases, (
        SELECT COALESCE(SUM(amount), 0) AS sum
        FROM gear_bank_transfers
        WHERE source = $1
    ) AS transfers_out, (
        SELECT COALESCE(SUM(amount), 0) AS sum
        FROM gear_bank_transfers
        WHERE target = $1
    ) AS transfers_in;
    RETURN balance;
END
$$ LANGUAGE plpgsql;

DROP FUNCTION gear_item_current_cost(VARCHAR(255));
CREATE OR REPLACE FUNCTION gear_item_current_cost(VARCHAR(255)) RETURNS TABLE (personal_current INTEGER, company_current INTEGER, end_date TIMESTAMPTZ) AS $$
    BEGIN
        RETURN QUERY 
            SELECT gear_cost.personal, gear_cost.company, gear_cost.end_date 
                FROM gear_cost 
                WHERE 
                    gear_cost.class = $1
                    AND (
                        (NOW() > start_date AND NOW() < gear_cost.end_date)
                        OR (start_date is NULL AND gear_cost.end_date is NULL)
                    )
                ORDER BY priority DESC LIMIT 1;
    END
$$ LANGUAGE plpgsql;

-- drop function to change signature
DROP FUNCTION gear_item_base_cost(VARCHAR(255));
CREATE OR REPLACE FUNCTION gear_item_base_cost(VARCHAR(255)) RETURNS TABLE (personal INTEGER, company INTEGER) AS $$
    DECLARE result INTEGER;
    BEGIN
        RETURN QUERY
            SELECT personal, company 
                FROM gear_cost
                WHERE 
                    class = $1
                    AND start_date IS NULL
                    AND end_date IS NULL 
                ORDER BY priority DESC LIMIT 1;
    END
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION gear_verify_purchase_commit()
    RETURNS trigger AS
$func$
BEGIN
    IF NOT NEW.global THEN 
        IF NOT gear_has_funds(NEW.member, NEW.personal * NEW.quantity) THEN
            RAISE EXCEPTION 'Insufficient funds';
        END IF;
    END IF;
    RETURN NEW;
END
$func$ LANGUAGE plpgsql;

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
