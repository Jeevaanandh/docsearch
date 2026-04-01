use pdf_extract;
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use dirs;


pub fn extract_pdf(file: &str) -> Result<(), Box<dyn std::error::Error>> {

    //This gets the directory to put the embedding model in
    let cache_dir = dirs::home_dir()
    .unwrap()
    .join(".cache/fastembed");

    //Creating the model
    let mut model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::AllMiniLML6V2)
            .with_show_download_progress(true)
            .with_cache_dir(cache_dir),
    )?;


    

    

    //Extracting the text contents from the PDF
    let text = pdf_extract::extract_text(file)?; 
    let text = text.replace('\u{00A0}', " ");

    let chunks: Vec<String> = text
        .split("\n\n")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    let embeddings = model.embed(chunks, None)?;  

    println!("Embeddings length: {}", embeddings.len());
    println!("Embedding dimension: {}", embeddings[0].len());

    Ok(())
}