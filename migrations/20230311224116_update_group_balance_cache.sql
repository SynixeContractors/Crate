-- group balance trigger
CREATE TRIGGER garage_purchases_group_balance_cache_trigger
    AFTER INSERT OR UPDATE ON garage_purchases
    FOR EACH ROW
    EXECUTE PROCEDURE gear_update_bank_balance_cache();

-- group balance trigger for delete
CREATE TRIGGER garage_purchases_group_balance_cache_trigger_delete
    AFTER DELETE ON garage_purchases
    FOR EACH ROW
    EXECUTE PROCEDURE gear_update_bank_balance_cache_on_delete();


CREATE OR REPLACE FUNCTION gear_group_balance() RETURNS integer as $$
DECLARE balance integer;
BEGIN
-- Update Group
    SELECT (
        gear_bank_deposits.sum - garage_purchases.sum - gear_bank_purchases.sum
    ) INTO balance
    FROM (
        SELECT COALESCE(SUM(amount), 0) AS sum
        FROM gear_bank_deposits
        WHERE member = '0'
    ) AS gear_bank_deposits, (
        SELECT COALESCE(SUM(cost), 0) AS sum
        FROM garage_purchases
    ) As garage_purchases, (
        SELECT COALESCE(SUM(cost * quantity), 0) AS sum
        FROM gear_bank_purchases
        WHERE member = '0' OR global = true
    ) AS gear_bank_purchases;
    RETURN balance;
END
$$ LANGUAGE plpgsql;


-- Update the bank balance cache
CREATE OR REPLACE FUNCTION gear_update_bank_balance_cache()
    RETURNS trigger AS
$func$
DECLARE balance integer;
BEGIN
    -- Update member
    SELECT gear_get_member_balance(NEW.member) INTO balance;
    INSERT INTO gear_bank_balance_cache (member, balance)
    VALUES (NEW.member, balance)
    ON CONFLICT (member) DO UPDATE SET balance = EXCLUDED.balance;

    -- Update group
    SELECT gear_group_balance() INTO balance;
    INSERT INTO gear_bank_balance_cache (member, balance)
    VALUES (0, balance)
    ON CONFLICT (member) DO UPDATE SET balance = EXCLUDED.balance;

    RETURN null;
END
$func$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION gear_update_bank_balance_cache_on_delete()
    RETURNS trigger AS
$func$
DECLARE balance integer;
BEGIN
    -- Update member
    SELECT gear_get_member_balance(OLD.member) INTO balance;
    INSERT INTO gear_bank_balance_cache (member, balance)
    VALUES (OLD.member, balance)
    ON CONFLICT (member) DO UPDATE SET balance = EXCLUDED.balance;

    -- Update group
    SELECT gear_group_balance() INTO balance;
    INSERT INTO gear_bank_balance_cache (member, balance)
    VALUES (0, balance)
    ON CONFLICT (member) DO UPDATE SET balance = EXCLUDED.balance;
    RETURN null;
END
$func$ LANGUAGE plpgsql;
