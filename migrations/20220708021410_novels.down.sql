-- Add down migration script here
drop table ddxsku_spider_novels;
drop index if exists idx_ddxsku_spider_novels_name_author;