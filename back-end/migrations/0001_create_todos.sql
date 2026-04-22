CREATE TABLE todos (
    id          TEXT      PRIMARY KEY,
    slug        TEXT      NOT NULL UNIQUE,
    title       TEXT      NOT NULL,
    description TEXT      NOT NULL,
    completed   BOOLEAN   NOT NULL DEFAULT FALSE
);
