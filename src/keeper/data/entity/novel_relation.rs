use sea_orm::prelude::*;

#[derive(Debug, PartialEq, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "novel_relations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub spider_kind_id: String,
    #[sea_orm(primary_key)]
    pub novel_id: i64,
    #[sea_orm(primary_key)]
    pub spider_novel_id: i64,
    #[sea_orm(default_value = 0)]
    pub score: i32,
}

#[derive(Debug, Copy, Clone, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        todo!()
    }
}

impl ActiveModelBehavior for ActiveModel {}
