-- Add migration script here
CREATE OR REPLACE FUNCTION add_cost_to_garage_purchase() RETURNS TRIGGER AS $$
DECLARE
    cost INTEGER;
BEGIN
    IF NEW.cost IS NULL THEN
        SELECT cost INTO cost FROM garage_shop WHERE id = NEW.id;
        NEW.cost = cost;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER add_cost_to_garage_purchase
BEFORE INSERT ON garage_purchases
FOR EACH ROW EXECUTE PROCEDURE add_cost_to_garage_purchase();
