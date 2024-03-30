-- Add migration script here
CREATE TABLE surveys (
  id UUID PRIMARY KEY,
  title VARCHAR(255) NOT NULL,
  description TEXT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE survey_entries (
  survey_id UUID NOT NULL,
  member TEXT NOT NULL,
  option TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (survey_id) REFERENCES surveys(id) ON DELETE CASCADE,
  PRIMARY KEY (survey_id, member)
);

CREATE TABLE survey_options (
    survey_id UUID NOT NULL PRIMARY KEY,
    options TEXT[] NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (survey_id) REFERENCES surveys(id) ON DELETE CASCADE
);
