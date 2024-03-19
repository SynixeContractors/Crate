CREATE TABLE voting_polls (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    public_key TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE voting_keys (
    poll_id UUID NOT NULL REFERENCES voting_polls(id) ON DELETE CASCADE,
    staff TEXT NOT NULL,
    public_key TEXT,
    shard TEXT,
    private_key TEXT,
    PRIMARY KEY (poll_id, public_key)
);

CREATE TABLE voting_options (
    poll_id UUID NOT NULL REFERENCES voting_polls(id) ON DELETE CASCADE,
    id UUID NOT NULL DEFAULT uuid_generate_v4(),
    title TEXT NOT NULL,
    PRIMARY KEY (poll_id, id)
);

CREATE TABLE voting_ticket_box (
    poll_id UUID NOT NULL REFERENCES voting_polls(id) ON DELETE CASCADE,
    encrypted_ticket TEXT NOT NULL,
    PRIMARY KEY (poll_id, encrypted_ticket)
);

CREATE TABLE voting_vote_box (
    poll_id UUID NOT NULL REFERENCES voting_polls(id) ON DELETE CASCADE,
    encrypted_vote TEXT NOT NULL,
    PRIMARY KEY (poll_id, encrypted_vote)
);

CREATE TABLE voting_results (
    poll_id UUID PRIMARY KEY REFERENCES voting_polls(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    data JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
