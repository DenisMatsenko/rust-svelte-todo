CREATE TYPE user_role AS ENUM ('admin', 'editor', 'viewer');

ALTER TABLE users
    ADD COLUMN role user_role NOT NULL DEFAULT 'viewer';

-- Default admin user (password: "admin") — replace with a real account and delete this row.
INSERT INTO users (id, slug, full_name, email, password, role)
VALUES (
    'usr_admin_default',
    'admin',
    'Admin',
    'admin@example.com',
    '$argon2id$v=19$m=19456,t=2,p=1$LDVKWNWSCI3qicQqaM1MxA$e/97NhDuiclkRyJrca4+iqVHWDKAxjF4h4+TW7EDbIQ',
    'admin'
);
