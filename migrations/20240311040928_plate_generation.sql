-- Add migration script here
-- add column if not exists
ALTER TABLE garage_vehicles ADD COLUMN IF NOT EXISTS plate_template text;

-- generate a plate number from a template
-- the template will looks like one of: SYN-Z#, SNO-##, SRB-##
-- where # is an incrementing number, zero padded if there are more than one digit
CREATE OR REPLACE FUNCTION generate_plate(vehicle_id uuid) RETURNS text AS $$
DECLARE
    template text;
    plate_number text;
    variable integer := 0;
    matches text[];
    match text;
BEGIN
    SELECT plate_template INTO template FROM garage_shop WHERE id = vehicle_id;
    LOOP
        variable := variable + 1;
        IF variable > 1000 THEN
            RETURN NULL;
        END IF;
        matches := regexp_matches(template, '#+', 'g');
        plate_number := template;
        FOREACH match IN ARRAY matches
        LOOP
            plate_number := replace(plate_number, match, lpad(variable::text, length(match), '0'));
        END LOOP;
        EXIT WHEN NOT EXISTS (SELECT 1 FROM garage_vehicles WHERE plate = plate_number);
    END LOOP;
    RETURN plate_number;
END;
$$ LANGUAGE plpgsql;
