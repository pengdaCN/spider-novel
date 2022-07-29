use static_init::dynamic;

pub mod common;
pub mod ddxsku;
pub mod keeper;
pub mod spider;
pub mod webook;

#[dynamic]
static mut GEN: snowflake::SnowflakeIdGenerator = snowflake::SnowflakeIdGenerator::new(1, 1);
