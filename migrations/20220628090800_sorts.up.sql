-- Add up migration script here
create table if not exists sorts
(
    id               integer not null,
    created_at       text    not null,
    updated_at       text,
    name             text    not null,
    relation_kind_id text,
    relation_id      integer,
    primary key (id)
)