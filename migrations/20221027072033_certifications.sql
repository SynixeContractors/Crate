-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS certifications (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    link VARCHAR(255) NOT NULL,
    roles_required JSONB NOT NULL,
    roles_granted JSONB NOT NULL,
    valid_for INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS certifications_trials (
    id UUID PRIMARY KEY,
    certification_id UUID NOT NULL,
    trainee_id VARCHAR(128) NOT NULL,
    instructor_id VARCHAR(128) NOT NULL,
    notes TEXT NOT NULL,
    valid_until TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT certifications_trials_certification_id_fk FOREIGN KEY (certification_id) REFERENCES certifications (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS certifications_instructors (
    certification_id UUID NOT NULL,
    user_id VARCHAR(128) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (certification_id, user_id),
    CONSTRAINT certifications_instructors_certification_id_fk FOREIGN KEY (certification_id) REFERENCES certifications (id) ON DELETE CASCADE
);
