-- Add migration script here
CREATE TABLE gear_items_family (
    family VARCHAR(255) NOT NULL,
    class VARCHAR(255) NOT NULL,
    relation VARCHAR(32) NOT NULL,
    PRIMARY KEY (family, class, relation) 
);

CREATE INDEX items_family_family_relation_idx ON gear_items_family (family, relation);
