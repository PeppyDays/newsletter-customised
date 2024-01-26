CREATE TABLE subscription_tokens (
    token TEXT NOT NULL UNIQUE,
    subscriber_id UUID NOT NULL,
    issued_at TIMESTAMP WITH TIME ZONE NOT NULL,
    expired_at TIMESTAMP WITH TIME ZONE NOT NULL
);
