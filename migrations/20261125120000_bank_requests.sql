-- A member asking another to pay them, from `/bank request`
CREATE TABLE IF NOT EXISTS bank_requests (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    requester VARCHAR(128) NOT NULL,
    payer VARCHAR(128) NOT NULL,
    amount INTEGER NOT NULL CHECK (amount > 0),
    reason TEXT NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'paying', 'paid', 'declined')),
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS bank_requests_requester_idx ON bank_requests (requester, created DESC);
CREATE INDEX IF NOT EXISTS bank_requests_payer_idx ON bank_requests (payer, created DESC);
