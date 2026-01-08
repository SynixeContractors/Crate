-- Add migration script here
ALTER TABLE garage_colors DROP COLUMN textures;
ALTER TABLE garage_colors ADD COLUMN texture_source TEXT NOT NULL;
