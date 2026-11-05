create table account (
    id            uuid        primary key default uuidv7(),
    email         text        not null,
    password_hash text        not null,
    role          text        not null,
    created_at    timestamptz not null default now(),
    constraint account_email_key unique (email),
    constraint account_email_lowercase check (email = lower(email)),
    constraint account_email_length check (char_length(email) between 3 and 254),
    constraint account_role_name check (role ~ '^[a-z][a-z0-9_-]{0,31}$')
);

comment on column account.id is 'UUIDv7 (time-ordered for index locality); the service assigns it, the default covers other writers';
comment on column account.password_hash is 'Argon2id encoded hash, never the password';
comment on column account.role is 'Role name defined by the deploying project, not by this service';
