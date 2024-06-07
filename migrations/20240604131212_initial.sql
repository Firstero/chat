-- Add migration script here

-- create users table
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(255) NOT NULL,
    -- password argon2 hashed
    password_hash VARCHAR(97) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- create index on email
CREATE UNIQUE INDEX user_email_idx ON users(email);

-- create chat type, single, group, private_channel, public_channel
CREATE TYPE chat_type AS ENUM ('single', 'group', 'private_channel', 'public_channel');

-- create chat table
CREATE TABLE IF NOT EXISTS chat (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    type chat_type NOT NULL,
    -- user id list
    members BIGINT[] NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- create index on created_at desc
CREATE INDEX chat_created_at_idx ON chat(created_at DESC);

-- create message table
CREATE TABLE IF NOT EXISTS message (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chat(id),
    sender_id BIGINT NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    images TEXT[],
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 联合索引 chat_id, created_at desc
CREATE INDEX IF NOT EXISTS message_chat_id_created_at_idx ON message(chat_id, created_at DESC);
-- create index on sender_id, created_at desc
CREATE INDEX IF NOT EXISTS message_sender_id_created_at_idx ON message(sender_id, created_at DESC);



