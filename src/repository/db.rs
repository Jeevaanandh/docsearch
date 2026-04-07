use sqlx::{Result, SqlitePool};

#[derive(Debug, sqlx::FromRow)]
struct Rows {
    path: String,
    embedding: Vec<u8>,
}

pub async fn db_init() -> Result<SqlitePool> {
    let pool = SqlitePool::connect("sqlite://test.db?mode=rwc").await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS embeddings (
        path TEXT PRIMARY KEY,
        embedding blob
        )"#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

fn vec_to_bytes(vec: &Vec<f32>) -> Vec<u8> {
    vec.iter().flat_map(|f| f.to_le_bytes()).collect()
}

fn bytes_to_vec_f32(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| {
            let arr: [u8; 4] = chunk.try_into().unwrap();
            f32::from_le_bytes(arr) // or from_ne_bytes
        })
        .collect()
}

pub async fn add_embedding(pool: &SqlitePool, path: &str, embedding: &Vec<f32>) -> Result<()> {
    let bytes = vec_to_bytes(&embedding);

    sqlx::query("INSERT OR REPLACE INTO embeddings (path, embedding) VALUES (?, ?)")
        .bind(path)
        .bind(bytes)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn search_db(pool: &SqlitePool) -> Result<(Vec<String>, Vec<Vec<f32>>), sqlx::Error> {
    let rows: Vec<Rows> = sqlx::query_as("SELECT * FROM embeddings")
        .fetch_all(pool)
        .await?;

    let mut paths = Vec::with_capacity(rows.len());
    let mut embeddings = Vec::with_capacity(rows.len());

    for row in rows {
        paths.push(row.path);
        embeddings.push(bytes_to_vec_f32(&row.embedding));
    }

    Ok((paths, embeddings))
}

pub async fn get_paths(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as("SELECT path FROM embeddings")
        .fetch_all(pool)
        .await?;

    let paths = rows.into_iter().map(|(p,)| p).collect();

    Ok(paths)
}

pub async fn delete_path(path: &str, pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM embeddings WHERE path= ?")
        .bind(path)
        .execute(pool)
        .await?;

    Ok(())
}
