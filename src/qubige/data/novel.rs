use anyhow::Result;
use sqlx::{Pool, Sqlite};

pub async fn add(db: &mut Pool<Sqlite>) -> Result<super::Novel> {
    unimplemented!();
}

