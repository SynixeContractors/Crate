-- Add migration script here
CREATE TABLE gear_items (
    class VARCHAR(255) NOT NULL,
    roles TEXT,
    category VARCHAR(16),
    global BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY class
);

CREATE INDEX items_roles_idx ON gear_items roles;
CREATE INDEX items_category_idx ON gear_items category;
CREATE INDEX items_global_idx ON gear_items global;

-- cost
CREATE TABLE gear_cost (
    class VARCHAR(255) NOT NULL,
    cost SERIAL NOT NULL,
    start_date TIMESTAMP,
    end_date TIMESTAMP,
    PRIMARY KEY class
);

ADD CONSTRAINT fk_items_cost FOREIGN KEY (class) REFERENCES gear_items(class); 

CREATE INDEX cost_class_idx ON gear_cost class;
CREATE INDEX cost_cost_idx ON gear_cost cost;
CREATE INDEX cost_start_date_idx ON gear_cost start_date;
CREATE INDEX cost_end_date_idx ON gear_cost end_date;

-- Loadouts
CREATE TABLE gear_loadouts (
    member VARCHAR(128),
    loadout TEXT NOT NULL,
    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY member
);

CREATE OR REPLACE FUNCTION gear_loadouts_update_created_on_update() RETURNS TRIGGER AS $$
BEGIN
    NEW.created = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER gear_loadouts_update_created_on_update
BEFORE UPDATE ON gear_loadouts
FOR EACH ROW
EXECUTE PROCEDURE gear_loadouts_update_created_on_update();

CREATE TABLE gear_loadouts_log (
    member VARCHAR(128) NOT NULL,
    loadout TEXT NOT NULL,
    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (member, created)
);

CREATE INDEX old_loadouts_member_idx ON gear_loadouts_log member;

-- Move a row to gear_loadouts_log when updated in loadouts
CREATE OR REPLACE FUNCTION gear_move_to_old_loadouts()
    RETURNS trigger AS
$func$
BEGIN
    INSERT INTO gear_loadouts_log (member, loadout, created)
    VALUES (OLD.member, OLD.loadout, OLD.created);
    DELETE FROM gear_loadouts_log WHERE member = NEW.member AND created < (SELECT created FROM gear_loadouts_log WHERE member = NEW.member ORDER BY created DESC LIMIT 1 OFFSET 1000);
    RETURN NEW;
END
$func$ LANGUAGE plpgsql;

CREATE TRIGGER gear_move_to_old_loadouts
BEFORE UPDATE ON gear_loadouts
FOR EACH ROW
EXECUTE PROCEDURE gear_move_to_old_loadouts();

-- Bank
CREATE TABLE gear_bank_balance_cache (
    member VARCHAR(128) NOT NULL,
    balance BIGINT NOT NULL,
    PRIMARY KEY member
);

CREATE TABLE gear_bank_purchases (
    member VARCHAR(128) NOT NULL,
    class VARCHAR(16) NOT NULL,
    quantity INTEGER NOT NULL,
    cost INTEGER NOT NULL,
    global BOOLEAN NOT NULL DEFAULT false,
    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (member, class, created)
);

CREATE INDEX purchases_member_idx ON gear_bank_purchases member;
CREATE INDEX purchases_class_idx ON gear_bank_purchases class;
CREATE INDEX purchases_global_idx ON gear_bank_purchases global;

CREATE TABLE gear_bank_deposits (
    member VARCHAR(128) NOT NULL,
    amount INTEGER NOT NULL,
    reason VARCHAR(50) NOT NULL,
    id UUID NOT NULL DEFAULT uuid_generate_v4(),
    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (member, id, created)
);

CREATE INDEX deposits_member_idx ON gear_bank_deposits member;
CREATE INDEX deposits_reason_idx ON gear_bank_deposits reason;
CREATE INDEX deposits_id_idx ON gear_bank_deposits id;

CREATE TABLE gear_bank_transfers (
    source VARCHAR(128) NOT NULL,
    target VARCHAR(128) NOT NULL,
    amount INTEGER NOT NULL,
    reason VARCHAR(255) NOT NULL,
    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (source, target, created)
);

CREATE INDEX transfers_source_idx ON gear_bank_transfers source;
CREATE INDEX transfers_target_idx ON gear_bank_transfers target;
CREATE INDEX transfers_reason_idx ON gear_bank_transfers reason;

-- Retrieve the balance of a member
-- Take into account the deposits, purchases, and transfers
-- Do not include global purchases
CREATE OR REPLACE FUNCTION gear_get_member_balance(VARCHAR(128)) RETURNS integer AS $$
DECLARE balance integer;
BEGIN
    SELECT (
        gear_bank_deposits.sum - gear_bank_purchases.sum - transfers_out.sum + transfers_in.sum
    ) INTO balance
    FROM (
        SELECT COALESCE(SUM(amount), 0) AS sum
        FROM gear_bank_deposits
        WHERE member = $1
    ) AS gear_bank_deposits, (
        SELECT COALESCE(SUM(cost * quantity), 0) AS sum
        FROM gear_bank_purchases
        WHERE member = $1 AND global = false
    ) AS gear_bank_purchases, (
        SELECT COALESCE(SUM(amount), 0) AS sum
        FROM gear_bank_transfers
        WHERE AND source = $1
    ) AS transfers_out, (
        SELECT COALESCE(SUM(amount), 0) AS sum
        FROM gear_bank_transfers
        WHERE target = $1
    ) AS transfers_in;
    RETURN balance;
END
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION gear_has_funds(VARCHAR(128), integer) RETURNS boolean AS $$
DECLARE balance integer;
BEGIN
    SELECT gear_get_member_balance($1) INTO balance;
    RETURN balance >= $3;
END
$$ LANGUAGE plpgsql;


-- Verify a transaction
-- Check that the source member has enough balance
-- Check that the source and target members are not the same
-- Check that the amount is positive
CREATE OR REPLACE FUNCTION gear_verify_transfer(varchar(128), VARCHAR(128), integer) RETURNS boolean AS $$
DECLARE balance integer;
BEGIN
    SELECT gear_get_member_balance($1, $2) INTO balance;
    RETURN balance >= $3 AND $1 <> $2 AND $3 > 0;
END
$$ LANGUAGE plpgsql;

-- Verify the transfer before committing it
CREATE OR REPLACE FUNCTION gear_verify_transfer_commit()
    RETURNS trigger AS
$func$
BEGIN
    IF NOT gear_verify_transfer(NEW.source, NEW.target, NEW.amount) THEN
        RAISE EXCEPTION 'Insufficient funds';
    END IF;
    RETURN NEW;
END
$func$ LANGUAGE plpgsql;

-- Transfer trigger
CREATE TRIGGER gear_bank_transfers_trigger
    BEFORE INSERT ON gear_bank_transfers
    FOR EACH ROW
    EXECUTE PROCEDURE gear_verify_transfer_commit();

-- Check gear_has_funds before performing the purchase
CREATE OR REPLACE FUNCTION gear_verify_purchase_commit()
    RETURNS trigger AS
$func$
BEGIN
    IF NOT NEW.global THEN 
        IF NOT gear_has_funds(NEW.member, NEW.cost * NEW.quantity) THEN
            RAISE EXCEPTION 'Insufficient funds';
        END IF;
    END IF;
    RETURN NEW;
END
$func$ LANGUAGE plpgsql;

CREATE TRIGGER gear_bank_purchases_trigger
    BEFORE INSERT ON gear_bank_purchases
    FOR EACH ROW
    EXECUTE PROCEDURE gear_verify_purchase_commit();

-- Update the bank balance cache
CREATE OR REPLACE FUNCTION gear_update_bank_balance_cache()
    RETURNS trigger AS
$func$
DECLARE balance integer;
BEGIN
    SELECT gear_get_member_balance(NEW.member) INTO balance;
    INSERT INTO gear_bank_balance_cache (member, balance)
    VALUES (NEW.member, balance)
    ON CONFLICT (member) DO UPDATE SET balance = EXCLUDED.balance;
    RETURN null;
END
$func$ LANGUAGE plpgsql;

-- Update the balance cache when a purchase is made
CREATE TRIGGER gear_bank_purchases_update_balance_cache
    AFTER INSERT OR UPDATE ON gear_bank_purchases
    FOR EACH ROW
    EXECUTE PROCEDURE gear_update_bank_balance_cache();

-- Update the balance cache when a deposit is made
CREATE TRIGGER gear_bank_deposits_update_balance_cache
    AFTER INSERT OR UPDATE ON gear_bank_deposits
    FOR EACH ROW
    EXECUTE PROCEDURE gear_update_bank_balance_cache();

-- Update the balance cache when a transfer is made
CREATE OR REPLACE FUNCTION gear_update_transfer_balance_cache()
    RETURNS trigger AS
$func$
DECLARE balance integer;
BEGIN
    SELECT gear_get_member_balance(NEW.source) INTO balance;
    INSERT INTO gear_bank_balance_cache (member, balance)
    VALUES (NEW.source, balance)
    ON CONFLICT (member) DO UPDATE SET balance = EXCLUDED.balance;
    SELECT gear_get_member_balance(NEW.target) INTO balance;
    INSERT INTO gear_bank_balance_cache (member, balance)
    VALUES (NEW.target, balance)
    ON CONFLICT (member) DO UPDATE SET balance = EXCLUDED.balance;
    RETURN NULL;
END
$func$ LANGUAGE plpgsql;

-- Update the balance cache when a transfer is made
CREATE TRIGGER gear_bank_transfers_update_balance_cache_source
    AFTER INSERT OR UPDATE ON gear_bank_transfers
    FOR EACH ROW
    EXECUTE PROCEDURE gear_update_transfer_balance_cache();
CREATE TRIGGER bank_transfers_update_balance_cache_target
    AFTER INSERT OR UPDATE ON gear_bank_transfers
    FOR EACH ROW
    EXECUTE PROCEDURE gear_update_transfer_balance_cache();

-- Locker
CREATE TABLE gear_locker (
    member VARCHAR(128) NOT NULL,
    class VARCHAR(16) NOT NULL,
    quantity SERIAL NOT NULL,
    PRIMARY KEY (member, class)
);

CREATE INDEX locker_member_idx ON gear_locker (member);
CREATE INDEX locker_class_idx ON gear_locker (class);

CREATE TABLE gear_locker_log (
    member VARCHAR(128) NOT NULL,
    class VARCHAR(16) NOT NULL,
    quantity INTEGER NOT NULL,
    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (member, class, created)
);

CREATE INDEX locker_log_member_idx ON gear_locker_log (member);
CREATE INDEX locker_log_class_idx ON gear_locker_log (class);
CREATE INDEX locker_log_created_idx ON gear_locker_log (created);

-- Update the gear_locker to reflect the gear_locker log
CREATE OR REPLACE FUNCTION gear_locker_update(VARCHAR(128), VARCHAR(16)) RETURNS void AS $$
DECLARE stored integer;
BEGIN
    SELECT COALESCE(SUM(quantity), 0) INTO stored FROM gear_locker_log WHERE member = $1 AND class = $2;
    IF stored = 0 THEN
        DELETE FROM gear_locker WHERE member = $1 AND class = $2;
    ELSE
        INSERT INTO gear_locker (member, class, quantity)
            VALUES ($1, $2, stored)
            ON CONFLICT (member, class)
            DO UPDATE SET quantity = stored;
    END IF;
    RETURN;
END;
$$ LANGUAGE plpgsql;

-- Update the gear_locker when the gear_locker log is updated
CREATE OR REPLACE FUNCTION gear_locker_log_insert_check()
    RETURNS TRIGGER AS
$func$
DECLARE stored integer;
BEGIN
    IF NEW.quantity < 0 THEN
        SELECT COALESCE(SUM(quantity), 0) INTO stored FROM gear_locker WHERE member = NEW.member AND class = NEW.class;
        IF stored + NEW.quantity < 0 THEN
            RAISE EXCEPTION 'Locker quantity cannot be negative';
        END IF;
    END IF;
    RETURN NEW;
END;
$func$ LANGUAGE plpgsql;

CREATE TRIGGER gear_locker_log_insert_check
BEFORE INSERT OR UPDATE ON gear_locker_log
FOR EACH ROW
EXECUTE PROCEDURE gear_locker_log_insert_check();

CREATE OR REPLACE FUNCTION gear_locker_update_upsert()
    RETURNS TRIGGER AS
$func$
BEGIN
    PERFORM gear_locker_update(NEW.member, NEW.class);
    RETURN new;
END;
$func$ LANGUAGE plpgsql;

CREATE TRIGGER gear_locker_update_upsert
AFTER INSERT OR UPDATE ON gear_locker_log
FOR EACH ROW
EXECUTE PROCEDURE gear_locker_update_upsert();

CREATE OR REPLACE FUNCTION gear_locker_update_delete()
    RETURNS TRIGGER AS
$func$
BEGIN
    PERFORM gear_locker_update(OLD.member, OLD.class);
    return null;
END;
$func$ LANGUAGE plpgsql;

CREATE TRIGGER gear_locker_update_delete
AFTER DELETE ON gear_locker_log
FOR EACH ROW
EXECUTE PROCEDURE gear_locker_update_delete();

CREATE OR REPLACE FUNCTION gear_add_purchase_to_locker()
    RETURNS trigger AS
$func$
BEGIN
    INSERT INTO gear_locker_log (member, class, quantity, created)
    VALUES (NEW.member, NEW.class, NEW.quantity, NEW.created);
    RETURN null;
END
$func$ LANGUAGE plpgsql;

-- Update the balance cache when a purchase is made
CREATE TRIGGER gear_bank_purchases_add_to_locker
    AFTER INSERT ON gear_bank_purchases
    FOR EACH ROW
    EXECUTE PROCEDURE gear_add_purchase_to_locker();

CREATE OR REPLACE FUNCTION gear_update_bank_balance_cache_on_delete()
    RETURNS trigger AS
$func$
DECLARE balance integer;
BEGIN
    SELECT gear_get_member_balance(OLD.member) INTO balance;
    INSERT INTO gear_bank_balance_cache (member, balance)
    VALUES (OLD.member, balance)
    ON CONFLICT (member) DO UPDATE SET balance = EXCLUDED.balance;
    RETURN null;
END
$func$ LANGUAGE plpgsql;

-- Update the balance cache when a transfer is made
CREATE OR REPLACE FUNCTION gear_update_transfer_balance_cache_on_delete()
    RETURNS trigger AS
$func$
DECLARE balance integer;
BEGIN
    SELECT gear_get_member_balance(OLD.source) INTO balance;
    INSERT INTO gear_bank_balance_cache (member, balance)
    VALUES (OLD.source, balance)
    ON CONFLICT (member) DO UPDATE SET balance = EXCLUDED.balance;
    SELECT gear_get_member_balance(OLD.target) INTO balance;
    INSERT INTO gear_bank_balance_cache (member, balance)
    VALUES (OLD.target, balance)
    ON CONFLICT (member) DO UPDATE SET balance = EXCLUDED.balance;
    RETURN NULL;
END
$func$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS gear_bank_purchases_update_balance_cache_on_delete on gear_bank_purchases;
CREATE TRIGGER gear_bank_purchases_update_balance_cache_on_delete
    AFTER DELETE ON gear_bank_purchases
    FOR EACH ROW
    EXECUTE PROCEDURE gear_update_bank_balance_cache_on_delete();

-- Update the balance cache when a transfer is made
DROP TRIGGER IF EXISTS gear_bank_transfers_update_balance_cache_source_on_delete on gear_bank_transfers;
CREATE TRIGGER gear_bank_transfers_update_balance_cache_source_on_delete
    AFTER DELETE ON gear_bank_transfers
    FOR EACH ROW
    EXECUTE PROCEDURE gear_update_transfer_balance_cache_on_delete();

DROP TRIGGER IF EXISTS gear_bank_transfers_update_balance_cache_target_on_delete on gear_bank_transfers;
CREATE TRIGGER gear_bank_transfers_update_balance_cache_target_on_delete
    AFTER DELETE ON gear_bank_transfers
    FOR EACH ROW
    EXECUTE PROCEDURE gear_update_transfer_balance_cache_on_delete();
