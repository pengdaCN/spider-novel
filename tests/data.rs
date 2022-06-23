
#[test]
fn get_data() {
    use sqlx::sqlite::SqlitePoolOptions;

    let pool = SqlitePoolOptions::new()
        .connect("data.db").await.unwrap();

    sqlx::migrate!()

}