use sea_orm::prelude::*;

#[allow(unused_imports)]
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "generic_spider_sorts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    #[sea_orm(primary_key)]
    pub spider_id: String,
    #[sea_orm(primary_key)]
    pub name: String,
    pub link: String,
}

#[derive(Debug, Copy, Clone, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        todo!()
    }
}

impl ActiveModelBehavior for ActiveModel {}
