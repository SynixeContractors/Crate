-- Add migration script here
CREATE TABLE IF NOT EXISTS garage_colors (
    id UUID,
    name VARCHAR(32),
    textures TEXT[],
    PRIMARY KEY (id, name),
    CONSTRAINT garage_colors_id FOREIGN KEY (id) REFERENCES garage_shop (id) ON DELETE CASCADE
);
