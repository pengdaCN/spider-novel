-- Add up migration script here
create table if not exists ddxsku_spider_novels
(
    id           integer not null,
    raw_id       text    not null,
    name         text    not null,
    author       text    not null,
    raw_link     text    not null,
    section_link text    not null,
    primary key (id)
);

drop index if exists idx_ddxsku_spider_novels_name_author;
create unique index idx_ddxsku_spider_novels_name_author on ddxsku_spider_novels (name, author);