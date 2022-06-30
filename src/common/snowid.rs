
use snowflake::SnowflakeIdGenerator;
use tokio::sync::Mutex;

static mut GEN: Option<Mutex<SnowflakeIdGenerator>> = None;

pub fn set(machine_id: i32, node_id: i32) {
    unsafe {
        GEN = Some(Mutex::new(SnowflakeIdGenerator::new(machine_id, node_id)));
    }
}

pub async fn id() -> i64 {
    unsafe {
        if let Some(gen) = GEN.as_ref() {
            let mut gen = gen.lock().await;

            gen.generate()
        } else {
            panic!("snowflake is not init")
        }
    }
}