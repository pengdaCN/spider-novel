use sea_orm::prelude::*;

#[derive(Debug, PartialEq, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "ddxsku_spider_novels")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub raw_id: String,
    pub name: String,
    pub author: String,
    pub raw_link: String,
}

#[derive(Debug, Copy, Clone, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        todo!()
    }
}

impl ActiveModelBehavior for ActiveModel {}
