use sqlx::{any::install_default_drivers, sqlite};
use std::env;
use dotenvy::dotenv;
use uuid::Uuid;
use anyhow::anyhow;
use anyhow::Error;

#[tokio::main]
async fn main() {    
    dotenv().expect(".env file not found");
    let database_url: String = env::var("DATABASE_URL").expect(".env missing DATABASE_URL");
    install_default_drivers();
    let pool = sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await.expect("create pool");
    sqlx::migrate!().run(&pool).await.expect("migrate");
    let mut tx = pool.begin().await.unwrap();
    let id = insert(&mut tx, "book one").await;
    println!("id: {}", id.unwrap());
}


pub async fn insert(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    name: &str
) -> Result<String, Error> {
    let book_id = Uuid::new_v4().to_string();
    let r = sqlx::query!(
        r#"
        INSERT INTO Book 
        (book_id, name)
        VALUES
        ($1, $2)
        "#,
        book_id,
        name
    )
    .execute(&mut **tx)
    .await;

    match r {
        Ok(qr) => {
            if qr.rows_affected() == 1 {
                return Ok(book_id);
            } else {
                return Err(anyhow!("Insert did not return 1 row affected:{}",qr.rows_affected()));
            }
        }
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;
    use super::*;

    #[sqlx::test(migrations = "migrations/sqlite")]
    async fn test_insert(pool: SqlitePool) -> sqlx::Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;
        let id = insert(&mut tx, "book one").await;
        assert_eq!(&id.is_ok(), &true);

        Ok(())
    }
}
