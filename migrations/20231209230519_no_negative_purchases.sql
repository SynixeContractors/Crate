-- Add migration script here
CREATE OR REPLACE FUNCTION gear_bank_purchases_not_negative() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.quantity < 0 THEN
        RAISE EXCEPTION 'Cannot purchase negative quantity of gear';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER gear_bank_purchases_not_negative
BEFORE INSERT OR UPDATE ON gear_bank_purchases
FOR EACH ROW EXECUTE PROCEDURE gear_bank_purchases_not_negative();
