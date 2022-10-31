-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS certifications (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    link VARCHAR(255) NOT NULL,
    roles_required VARCHAR(128) ARRAY NOT NULL,
    roles_granted VARCHAR(128) ARRAY NOT NULL,
    valid_for INTEGER NOT NULL,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS certifications_trials (
    id UUID PRIMARY KEY,
    certification UUID NOT NULL,
    trainee VARCHAR(128) NOT NULL,
    instructor VARCHAR(128) NOT NULL,
    notes TEXT NOT NULL,
    valid_until TIMESTAMPTZ,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT certifications_trials_certification FOREIGN KEY (certification) REFERENCES certifications (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS certifications_instructors (
    certification UUID NOT NULL,
    member VARCHAR(128) NOT NULL,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (certification, member),
    CONSTRAINT certifications_instructors_certification FOREIGN KEY (certification) REFERENCES certifications (id) ON DELETE CASCADE
);
