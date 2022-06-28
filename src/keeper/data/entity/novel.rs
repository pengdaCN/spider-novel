use sea_orm::prelude::*;

#[derive(Debug, PartialEq, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "novels")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    // 小说名
    pub name: String,
    // 记录创建时间
    pub created_at: DateTimeUtc,
    // 记录更新时间
    pub updated_at: DateTimeUtc,
    // 封面地址
    pub cover: Option<String>,
    // 作者名
    pub author: String,
    // 小说上次更新时间
    pub last_updated_at: Option<DateTimeUtc>,
    // 最近更新的小说章节
    pub last_section: Option<i64>,
}

#[derive(Debug, Copy, Clone, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        todo!()
    }
}

impl ActiveModelBehavior for ActiveModel {}
