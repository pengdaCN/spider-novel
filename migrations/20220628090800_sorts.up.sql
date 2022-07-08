-- Add up migration script here
create table if not exists ddxsku_spider_sorts
(
    id   integer not null,
    name text    not null,
    link text    not null,
    primary key (id)
)