-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS certifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    link VARCHAR(255) NOT NULL,
    roles_required VARCHAR(128) ARRAY NOT NULL,
    roles_granted VARCHAR(128) ARRAY NOT NULL,
    valid_for INTEGER,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS certifications_trials (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    certification UUID NOT NULL,
    trainee VARCHAR(128) NOT NULL,
    instructor VARCHAR(128) NOT NULL,
    notes TEXT NOT NULL,
    passed BOOLEAN NOT NULL,
    valid_for INTEGER,
    valid_until TIMESTAMPTZ GENERATED ALWAYS AS (CASE WHEN valid_for IS NULL THEN NULL ELSE created AT TIME ZONE 'UTC' + make_interval(days => valid_for) END) STORED,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT certifications_trials_certification FOREIGN KEY (certification) REFERENCES certifications (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS certifications_instructors (
    certification UUID NOT NULL,
    member VARCHAR(128) NOT NULL,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (certification, member),
    CONSTRAINT certifications_instructors_certification FOREIGN KEY (certification) REFERENCES certifications (id) ON DELETE CASCADE
);
