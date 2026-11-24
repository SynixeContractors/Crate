-- garage_shop.plate_template is read by generate_plate() and the shop queries, but was only ever added
-- by hand: 20240311040928_plate_generation added it to garage_vehicles instead. A no-op where it exists.
ALTER TABLE garage_shop ADD COLUMN IF NOT EXISTS plate_template TEXT;
