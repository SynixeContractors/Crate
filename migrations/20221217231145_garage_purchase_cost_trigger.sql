-- Add migration script here
CREATE OR REPLACE FUNCTION add_cost_to_garage_purchase() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.cost IS NULL THEN
        NEW.cost = (SELECT cost FROM garage_shop WHERE id = NEW.id);
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER add_cost_to_garage_purchase
BEFORE INSERT ON garage_purchases
FOR EACH ROW EXECUTE PROCEDURE add_cost_to_garage_purchase();
