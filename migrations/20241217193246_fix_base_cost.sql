-- Add migration script here
DROP FUNCTION gear_item_base_cost(VARCHAR(255));
CREATE OR REPLACE FUNCTION gear_item_base_cost(VARCHAR(255)) RETURNS TABLE (personal INTEGER, company INTEGER) AS $$
    DECLARE result INTEGER;
    BEGIN
        RETURN QUERY
            SELECT gear_cost.personal, gear_cost.company 
                FROM gear_cost
                WHERE 
                    class = $1
                    AND start_date IS NULL
                    AND end_date IS NULL 
                ORDER BY priority DESC LIMIT 1;
    END
$$ LANGUAGE plpgsql;
