CREATE TABLE subscribers (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    subscribed_at TIMESTAMP WITH TIME ZONE NOT NULL
);
