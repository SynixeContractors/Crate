-- Add migration script here
CREATE TABLE gear_pretty (
    class VARCHAR(255) NOT NULL,
    pretty VARCHAR(255) NOT NULL,
    PRIMARY KEY (class)
);
