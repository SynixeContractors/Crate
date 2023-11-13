-- Add migration script here
CREATE TABLE missions_member_count(
    member VARCHAR(128) NOT NULL,
    count INT NOT NULL,
    PRIMARY KEY (member)
);

CREATE OR REPLACE FUNCTION missions_member_count_increment() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.id != '00000000-0000-0000-0000-000000000000' THEN
        INSERT INTO missions_member_count (member, count) VALUES (NEW.member, 1)
        ON CONFLICT (member) DO UPDATE SET count = missions_member_count.count + 1;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER missions_member_count_increment
AFTER INSERT ON gear_bank_deposits
FOR EACH ROW
EXECUTE PROCEDURE missions_member_count_increment();
