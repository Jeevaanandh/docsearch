use pdf_extract;
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use dirs;

pub fn extract_pdf(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cache_dir = dirs::home_dir()
    .unwrap()
    .join(".cache/fastembed");

    let mut model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::AllMiniLML6V2)
            .with_show_download_progress(true)
            .with_cache_dir(cache_dir),
    )?;

    let text = pdf_extract::extract_text(file)?;  // no unwrap

    let chunks: Vec<String> = text
        .split("\n\n")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    let embeddings = model.embed(chunks, None)?;  // clean error handling

    println!("Embeddings length: {}", embeddings.len());
    println!("Embedding dimension: {}", embeddings[0].len());

    Ok(())
}