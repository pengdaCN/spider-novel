-- Add up migration script here
create table if not exists sort
(
    id                 integer not null,
    created_at         text    not null,
    updated_at         text,
    name               text,
    relation_spider_id text,
    relation_id        text,
    primary key (id, name)
);