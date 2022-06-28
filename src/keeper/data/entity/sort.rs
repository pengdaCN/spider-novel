use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sorts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub created_at: DateTimeUtc,
    pub updated_at: Option<DateTimeUtc>,
    pub name: String,
    pub relation_kind_id: Option<String>,
    pub relation_id: Option<i64>,
}

#[derive(Debug, Copy, Clone, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        todo!()
    }
}

impl ActiveModelBehavior for ActiveModel {}
