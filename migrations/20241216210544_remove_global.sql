-- Add migration script here
ALTER TABLE gear_items DROP COLUMN global;

CREATE OR REPLACE FUNCTION gear_verify_purchase_commit()
    RETURNS trigger AS
$func$
BEGIN
    IF NOT gear_has_funds(NEW.member, NEW.personal * NEW.quantity) THEN
        RAISE EXCEPTION 'Insufficient funds';
    END IF;
    RETURN NEW;
END
$func$ LANGUAGE plpgsql;
