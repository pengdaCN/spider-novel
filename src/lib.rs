use static_init::dynamic;

pub mod keeper;
pub mod qubige;
pub mod spider;

#[dynamic]
static mut GEN: snowflake::SnowflakeIdGenerator = { snowflake::SnowflakeIdGenerator::new(1, 1) };
