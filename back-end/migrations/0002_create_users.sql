CREATE TABLE users (
    id          TEXT      PRIMARY KEY,
    slug        TEXT      NOT NULL UNIQUE,
    full_name   TEXT      NOT NULL,
    email       TEXT      NOT NULL UNIQUE,
    password    TEXT      NOT NULL
);
