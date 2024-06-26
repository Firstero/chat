-- Add migration script here

-- workspace for users
create table workspaces (
    id bigserial primary key,
    name varchar(255) not null unique,
    owner_id bigint not null,
    created_at timestamptz default current_timestamp
);

alter table users add column ws_id bigint references workspaces(id) not null;
-- add super user and super workspace
insert into workspaces (id, name, owner_id) values (0, 'super', 0);
insert into users (id, fullname, email, ws_id, password_hash) values (0, 'super', 'super@none.org', 0, '');
COMMIT;
-- alter workspaces owner_id column reference to users id column
alter table workspaces add constraint workspaces_owner_id_fkey foreign key (owner_id) references users(id);