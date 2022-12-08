-- Add migration script here
ALTER TABLE gear_bank_balance_cache ALTER COLUMN balance TYPE integer USING balance::integer;
