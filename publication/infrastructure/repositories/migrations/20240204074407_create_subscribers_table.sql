CREATE TYPE EMAIL_VERIFICATION_STATUS AS ENUM ('Unverified', 'Valid', 'Invalid');

CREATE TABLE subscribers (
    id UUID PRIMARY KEY,
    email_address TEXT NOT NULL UNIQUE,
    email_verification_status EMAIL_VERIFICATION_STATUS NOT NULL,
    name TEXT NOT NULL
);
