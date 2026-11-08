-- Advanced Equipment devices (laptops, flash drives, etc) can move between missions, e.g. carried
-- in someone's inventory from one campaign's area of operations to another's, so they are no
-- longer scoped to the campaign they were first saved under. Each device keeps its own id globally.
ALTER TABLE campaigns_ae3_devices DROP CONSTRAINT campaigns_ae3_devices_pkey;
ALTER TABLE campaigns_ae3_devices DROP CONSTRAINT campaigns_ae3_devices_campaign_fkey;
ALTER TABLE campaigns_ae3_devices DROP COLUMN campaign;
ALTER TABLE campaigns_ae3_devices ADD PRIMARY KEY (id);
