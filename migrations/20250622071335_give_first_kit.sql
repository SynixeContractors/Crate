CREATE OR REPLACE FUNCTION give_first_kit(member_id VARCHAR(128), first_kit_id UUID) RETURNS VOID AS $$
DECLARE
    i record;
    kit JSONB;
BEGIN
    SELECT first_kit INTO kit FROM certifications_first_kit WHERE id = first_kit_id;
    IF kit IS NOT NULL THEN
        FOR i IN SELECT * FROM jsonb_each(kit) LOOP
            INSERT INTO gear_bank_purchases (member, class, quantity, personal, company, reason)
                VALUES (member_id, i.Key, i.Value::INTEGER, 0, (SELECT company_current + personal_current FROM gear_item_current_cost(i.Key)), 'first kit');
        END LOOP;
    END IF;
END;
$$ LANGUAGE plpgsql;
