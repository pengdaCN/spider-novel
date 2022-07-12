use anyhow::Result;
use sea_orm::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::{Condition, ConnectionTrait, QuerySelect, TransactionTrait};

use crate::common::snowid::id;
use crate::ddxsku::DATA_URL;
use crate::spider::{NovelID, SortID};

pub mod novel;
pub mod sort;

pub async fn add_or_recover<T: ConnectionTrait>(db: &T, name: &str, link: &str) -> Result<i64> {
    // 查询时候存在相同名字的分类
    let selector: Select<_> = sort::Entity::find();
    let x: Option<sort::Model> = selector
        .column(sort::Column::Id)
        .filter(sort::Column::Name.eq(name))
        .one(db)
        .await?;

    let id = match x {
        Some(x) => {
            let id = x.id;
            if x.link != link {
                let mut x: sort::ActiveModel = x.into();
                x.link = Set(String::from(link));

                let _ = x.update(db).await?;
            }

            id
        }
        None => {
            let id = id().await;

            let x: sort::ActiveModel = sort::Model {
                id,
                name: String::from(name),
                link: String::from(link),
            }
            .into();

            let _ = sort::Entity::insert(x).exec(db).await?;

            id
        }
    };

    Ok(id)
}

pub async fn sort_by_id(db: &DbConn, id: &SortID) -> Result<Option<sort::Model>> {
    let selector: Select<_> = sort::Entity::find();
    let x = selector
        .filter(sort::Column::Id.eq(<SortID as Into<i64>>::into(*id)))
        .one(db)
        .await?;

    Ok(x)
}

pub async fn clear_sort<T: ConnectionTrait>(db: &T) -> Result<()> {
    let _ = sort::Entity::delete_many().exec(db).await?;

    Ok(())
}

pub async fn sorts<T: ConnectionTrait>(db: &T) -> Result<Vec<sort::Model>> {
    let selector: Select<_> = sort::Entity::find();
    let x = selector.all(db).await?;

    Ok(x)
}

pub async fn add_or_recover_novel(
    db: &DbConn,
    name: &str,
    link: &str,
    section_link: &str,
    author: &str,
    raw_id: &str,
) -> Result<i64> {
    // 查询时候存在相同名字的小说
    let selector: Select<_> = novel::Entity::find();
    let novel: Option<novel::Model> = selector
        .filter(
            Condition::all()
                .add(novel::Column::Name.eq(name))
                .add(novel::Column::Author.eq(author)),
        )
        .one(db)
        .await?;

    let id = match novel {
        Some(x) => {
            let id = x.id;
            if x.raw_link != link || x.section_link != section_link || x.raw_id != raw_id {
                let mut novel: novel::ActiveModel = x.into();

                novel.raw_link = Set(String::from(link));
                novel.section_link = Set(String::from(section_link));
                novel.raw_id = Set(String::from(raw_id));

                let _ = novel.update(db).await?;
            }

            id
        }
        None => {
            let id = id().await;

            let x: novel::ActiveModel = novel::Model {
                id,
                raw_id: String::from(raw_id),
                name: String::from(name),
                author: String::from(author),
                raw_link: String::from(link),
                section_link: String::from(section_link),
            }
            .into();

            let _ = novel::Entity::insert(x).exec(db).await?;

            id
        }
    };

    Ok(id)
}

pub async fn novel_by_id(db: &DbConn, id: &NovelID) -> Result<Option<novel::Model>> {
    let x = novel::Entity::find_by_id(id.into()).one(db).await?;

    Ok(x)
}
