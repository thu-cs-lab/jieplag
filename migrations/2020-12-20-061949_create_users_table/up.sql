CREATE TABLE users (
    id SERIAL NOT NULL,
    user_name TEXT NOT NULL,
    salt BYTEA NOT NULL,
    password BYTEA NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (user_name)
);