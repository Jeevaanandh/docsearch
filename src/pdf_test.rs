use crate::repository::db::add_embedding;
use dirs;
use fastembed::{EmbeddingModel, Error, InitOptions, TextEmbedding};
use pdf_extract;
use sqlx::SqlitePool;

pub fn average_embedding(embeddings: &Vec<Vec<f32>>) -> Vec<f32> {
    let dim = embeddings[0].len();
    let mut avg: Vec<f32> = vec![0.0; dim];

    for i in 0..embeddings.len() {
        let emb = &embeddings[i];
        for j in 0..dim {
            avg[j] += emb[j];
        }
    }

    for i in 0..dim {
        avg[i] /= embeddings.len() as f32;
    }

    avg
}

pub fn get_embedding(text: &str) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
    //This gets the directory to put the embedding model in
    let cache_dir = dirs::home_dir().unwrap().join(".cache/fastembed");

    //Creating the model
    let mut model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::AllMiniLML6V2)
            .with_show_download_progress(true)
            .with_cache_dir(cache_dir),
    )?;

    let chunks: Vec<String> = text
        .split("\n\n")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    let embeddings = model.embed(chunks, None)?;

    Ok(embeddings)
}

pub async fn extract_pdf(file: &str, pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    //Extracting the text contents from the PDF
    let text = pdf_extract::extract_text(file)?;
    let text = text.replace('\u{00A0}', " ");

    if text.len() == 0 {
        return Ok(());
    }

    let embeddings = get_embedding(&text)?;

    //This has to be added to the db with the file name as the primary key.
    let avg_embeddings = average_embedding(&embeddings);

    add_embedding(&pool, file, &avg_embeddings).await?;

    Ok(())
}
