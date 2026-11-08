-- AE3 keeps a packed laptop/flash drive's state in a server-memory-only buffer (AE3_LAPTOP_ITEM,
-- AE3_ITEM) while it's an inventory item, so that data is lost on a mission/server restart even
-- though the device's own state in campaigns_ae3_devices is safe. This table tracks which inventory
-- item class a packed device currently belongs to, so the buffer can be rebuilt at mission start and
-- the item can still be redeployed. The row is removed once the item is redeployed and the device is
-- a live object again.
CREATE TABLE campaigns_ae3_packed_items (
    item_class VARCHAR NOT NULL PRIMARY KEY,
    device_id UUID NOT NULL REFERENCES campaigns_ae3_devices(id) ON DELETE CASCADE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
