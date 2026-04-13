use sqlx::{Result, SqlitePool};
use std::env;
use std::path::PathBuf;

#[derive(Debug, sqlx::FromRow)]
struct Rows {
    file: String,
    filepath: String,
    dir: String,
    embedding: Vec<u8>,
}

pub async fn db_init() -> Result<SqlitePool> {
    let home_dir = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE")) // Windows fallback
        .expect("Could not determine home directory");

    let db_path = PathBuf::from(home_dir).join("test.db");
    let connection_string = format!("sqlite://{}?mode=rwc", db_path.display());

    let pool = SqlitePool::connect(&connection_string).await?;

    //Modified the schema to fit the gloal test.db_init
    //file --- name of the file
    //filepath --- path of that file (used to open the file)
    //dir --- to store the directory the
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS embeddings (
        file TEXT,
        filepath TEXT PRIMARY KEY,
        dir TEXT,
        embedding blob
        )"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query("PRAGMA journal_mode = WAL")
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

//Modified for global test.db
pub async fn add_embedding(
    pool: &SqlitePool,
    file: &str,
    filepath: &str,
    embedding: &Vec<f32>,
    dir: &str,
) -> Result<(), sqlx::Error> {
    let bytes = vec_to_bytes(&embedding);

    sqlx::query(
        "INSERT OR REPLACE INTO embeddings (file, filepath, dir, embedding) VALUES (?, ?, ?, ?)",
    )
    .bind(file)
    .bind(filepath)
    .bind(dir)
    .bind(bytes)
    .execute(pool)
    .await?;

    Ok(())
}

//Modified for global test.db
pub async fn search_db(
    pool: &SqlitePool,
) -> Result<(Vec<String>, Vec<Vec<f32>>, Vec<String>), sqlx::Error> {
    let rows: Vec<Rows> = sqlx::query_as("SELECT * FROM embeddings")
        .fetch_all(pool)
        .await?;

    let mut files = Vec::with_capacity(rows.len());
    let mut filepaths = Vec::with_capacity(rows.len());
    let mut embeddings = Vec::with_capacity(rows.len());

    for row in rows {
        files.push(row.file);
        embeddings.push(bytes_to_vec_f32(&row.embedding));
        filepaths.push(row.filepath);
    }

    Ok((files, embeddings, filepaths))
}

//This function was modified to fit the global test.db
//It now returns files(vector of names of the files) and the filepaths(full paths of the files)
pub async fn get_paths(
    pool: &SqlitePool,
    current_dir: &str,
) -> Result<(Vec<String>, Vec<String>), sqlx::Error> {
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT file, filepath FROM embeddings WHERE dir = ?")
            .bind(current_dir)
            .fetch_all(pool)
            .await?;

    let mut files = Vec::with_capacity(rows.len());
    let mut filepaths = Vec::with_capacity(rows.len());

    for row in rows {
        files.push(row.0);
        filepaths.push(row.1);
    }
    Ok((files, filepaths))
}

//Modified for global test.db
pub async fn delete_path(filepath: &str, pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM embeddings WHERE filepath= ?")
        .bind(filepath)
        .execute(pool)
        .await?;

    Ok(())
}
