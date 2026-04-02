use sqlx::{SqlitePool, Result};


pub async fn db_init() -> Result<SqlitePool> {
    let pool = SqlitePool::connect("sqlite://test.db?mode=rwc").await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS embeddings (
        path TEXT PRIMARY KEY,
        embedding blob
        )"#
    )
    .execute(&pool)
    .await?;
    

    Ok(pool)

}


fn vec_to_bytes(vec: &Vec<f32>) -> Vec<u8> {
    vec.iter()
        .flat_map(|f| f.to_le_bytes())
        .collect()
}

pub async fn add_embedding(pool: &SqlitePool, path: &str, embedding: &Vec<f32>) -> Result<()>{
    let bytes = vec_to_bytes(&embedding);

    sqlx::query("INSERT OR REPLACE INTO embeddings (path, embedding) VALUES (?, ?)")
        .bind(path)
        .bind(bytes)
        .execute(pool)
        .await?;

    Ok(())

}

