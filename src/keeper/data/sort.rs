use anyhow::Result;
use chrono::{DateTime, Utc};
use sea_orm::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::{Condition, DatabaseConnection, TransactionTrait};

use crate::keeper::data::entity::sort;
use crate::spider::Sort;
use crate::GEN;

pub struct ListOpt<'a> {
    pub created_at_less_than: Option<&'a DateTime<Utc>>,
    pub relation_spider_id: Option<&'a str>,
}

pub async fn add_or_recover<'a>(
    db: &'a DatabaseConnection,
    id: &'a str,
    data: &Vec<Sort>,
) -> Result<()> {
    let data: Vec<sort::ActiveModel> = data
        .iter()
        .map(|x| {
            let mut gen = GEN.write();
            let now = Utc::now();
            sort::ActiveModel {
                id: Set(gen.generate()),
                created_at: Set(now.clone()),
                updated_at: Set(None),
                name: Set(x.name.clone()),
                relation_kind_id: Set(Some(id.into())),
                relation_id: Set(Some(x.id.into())),
            }
        })
        .collect();

    // TODO 使用引用有生命周期的问题
    let id = String::from(id);
    let _ = db
        .transaction(|tx| {
            Box::pin(async {
                let _ = sort::Entity::delete_many()
                    .filter(sort::Column::RelationKindId.eq(id))
                    .exec(tx)
                    .await?;
                let _ = sort::Entity::insert_many(data).exec(tx).await?;

                Ok::<(), DbErr>(())
            })
        })
        .await?;

    Ok(())
}

pub async fn list(db: &DatabaseConnection, opts: Option<ListOpt<'_>>) -> Result<Vec<sort::Model>> {
    let data: Vec<sort::Model> = {
        let cond = {
            let mut condition = Condition::all();

            if let Some(opts) = opts {
                if let Some(x) = opts.created_at_less_than {
                    condition = condition.add(sort::Column::CreatedAt.lt((*x).clone()));
                }

                if let Some(x) = opts.relation_spider_id {
                    condition = condition.add(sort::Column::RelationId.eq(x));
                }
            }

            condition
        };

        sort::Entity::find().filter(cond).all(db).await?
    };

    Ok(data)
}
