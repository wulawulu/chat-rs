-- this file is used for postgresql database initialization
-- create user table
CREATE TABLE IF NOT EXISTS users
(
    id         BIGSERIAL PRIMARY KEY,
    fullname   VARCHAR(64) NOT NULL,
    email      VARCHAR(64) NOT NULL,
    -- hashed argon2 password
    password   VARCHAR(64) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- create index for users for email
CREATE UNIQUE INDEX IF NOT EXISTS users_email_index ON users (email);

-- create chat type: single, group, private_channel, public_channel
CREATE TYPE chat_type AS ENUM ('single', 'group', 'private_channel', 'public_channel');

-- create chat table
CREATE TABLE IF NOT EXISTS chats
(
    id         BIGSERIAL PRIMARY KEY,
    name       VARCHAR(128) NOT NULL UNIQUE,
    type       chat_type NOT NULL,
    -- user id list
    members    BIGINT[] NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- create message table
CREATE TABLE IF NOT EXISTS messages
(
    id         BIGSERIAL PRIMARY KEY,
    chat_id    BIGINT NOT NULL,
    sender_id  BIGINT NOT NULL,
    content    TEXT NOT NULL,
    images    TEXT[],
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (chat_id) REFERENCES chats (id),
    FOREIGN KEY (sender_id) REFERENCES users (id)
);

-- create index for messages for chat_id and created_at order by created_at desc
CREATE INDEX IF NOT EXISTS messages_chat_id_created_at_index ON messages (chat_id, created_at DESC);

-- create index for messages for sender_id
CREATE INDEX IF NOT EXISTS messages_sender_id_index ON messages (sender_id);
