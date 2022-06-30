use crate::common::snowid::id;
use crate::ddxsku::DATA_URL;
use anyhow::Result;
use sea_orm::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::{Condition, QuerySelect, TransactionTrait};

pub mod sort;

pub async fn add_or_recover(db: &DbConn, name: &str, link: &str) -> Result<i64> {
    let id = id().await;

    // 查询时候存在相同名字的分类
    let selector: Select<_> = sort::Entity::find();
    let old_id: Option<i64> = selector
        .column(sort::Column::Id)
        .filter(
            Condition::all()
                .add(sort::Column::Name.eq(name))
                .add(sort::Column::SpiderId.eq(DATA_URL)),
        )
        .one(db)
        .await
        .and_then(|x: Option<sort::Model>| Ok(x.and_then(|x| Some(x.id))))?;
    let data = sort::ActiveModel {
        id: Set(id),
        spider_id: Set(String::from(DATA_URL)),
        name: Set(String::from(name)),
        link: Set(String::from(link)),
    };

    // 添加分类，若存在相同名字的则删除
    db.transaction(|tx| {
        Box::pin(async move {
            if let Some(id) = old_id {
                // let _ = sort::delete_by_id(id).exec(tx).await?;
            }

            let _ = sort::Entity::insert(data).exec(tx).await?;
            Ok::<(), DbErr>(())
        })
    })
    .await?;

    Ok(id)
}
