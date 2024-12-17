-- Add migration script here
CREATE OR REPLACE FUNCTION gear_group_balance() RETURNS integer as $$
DECLARE balance integer;
BEGIN
-- Update Group
    SELECT (
        gear_bank_deposits.sum - garage_purchases.sum - gear_bank_purchases.sum - gear_bank_deposits_salary.sum
    ) INTO balance
    FROM (
        SELECT COALESCE(SUM(amount), 0) AS sum
        FROM gear_bank_deposits
        WHERE member = '0'
    ) AS gear_bank_deposits, (
        SELECT COALESCE(SUM(amount), 0) AS sum
        FROM gear_bank_deposits
        WHERE member != '0'
    ) AS gear_bank_deposits_salary, (
        SELECT COALESCE(SUM(cost), 0) AS sum
        FROM garage_purchases
    ) As garage_purchases, (
        SELECT COALESCE(SUM(company * quantity), 0) AS sum
        FROM gear_bank_purchases
    ) AS gear_bank_purchases;
    RETURN balance;
END
$$ LANGUAGE plpgsql;
