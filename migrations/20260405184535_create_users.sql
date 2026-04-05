CREATE TABLE groups (
    id   SMALLINT    PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name VARCHAR(64) NOT NULL UNIQUE
);

CREATE TABLE users (
    id            INTEGER      PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    username      VARCHAR(128) NOT NULL UNIQUE,
    password_hash TEXT         NOT NULL,
    display_name  VARCHAR(256),
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),
    deactivated_at TIMESTAMPTZ
);

CREATE TABLE user_groups (
    user_id  INTEGER  NOT NULL REFERENCES users(id)  ON DELETE CASCADE,
    group_id SMALLINT NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, group_id)
);

INSERT INTO groups (name) VALUES ('admin'), ('staff');
