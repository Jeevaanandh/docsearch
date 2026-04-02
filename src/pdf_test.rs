use pdf_extract;
use fastembed::{EmbeddingModel, Error, InitOptions, TextEmbedding};
use dirs;


fn average_embedding(embeddings: &Vec<Vec<f32>>) -> Vec<f32> {
    let dim = embeddings[0].len();
    let mut avg = vec![0.0; dim];

    for emb in embeddings {
        for i in 0..dim {
            avg[i] += emb[i];
        }
    }

    for i in 0..dim {
        avg[i] /= embeddings.len() as f32;
    }

    avg

}


pub fn get_embedding(text: &str) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>>{

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


    let chunks: Vec<String> = text
        .split("\n\n")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();


    let embeddings= model.embed(chunks, None)?;

    Ok(embeddings)
        

}


pub fn extract_pdf(file: &str) -> Result<(), Box<dyn std::error::Error>> {


    //Extracting the text contents from the PDF
    let text = pdf_extract::extract_text(file)?; 
    let text = text.replace('\u{00A0}', " ");

    

    let embeddings = get_embedding(&text)?;

    println!("Embeddings length: {}", embeddings.len());
    println!("Embedding dimension: {}", embeddings[0].len());

    //This has to be added to the db with the file name as the primary key.
    let avg_embeddings= average_embedding(&embeddings);

    

    Ok(())
}