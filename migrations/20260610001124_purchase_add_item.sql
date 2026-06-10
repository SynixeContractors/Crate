-- Add migration script here
ALTER TABLE gear_bank_purchases
ADD COLUMN add_item BOOLEAN NOT NULL DEFAULT true;

CREATE OR REPLACE FUNCTION gear_add_purchase_to_locker()
    RETURNS trigger AS
$func$
BEGIN
    IF NEW.add_item THEN
        INSERT INTO gear_locker_log (member, class, quantity, created)
        VALUES (NEW.member, NEW.class, NEW.quantity, NEW.created);
    END IF;
    RETURN null;
END
$func$ LANGUAGE plpgsql;
