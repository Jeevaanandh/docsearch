use crate::repository::db::add_embedding;
use dirs;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use pdf_extract;
use pdf_oxide::PdfDocument;
use sqlx::SqlitePool;

use crate::embed::{average_embedding, get_embedding, process};

/*
pub async fn extract_pdf(
    cur_dir: &str,
    filename: &str,
    filepath: &str,
    pool: &SqlitePool,
) -> Result<(), Box<dyn std::error::Error>> {
    //Extracting the text contents from the PDF
    let text = pdf_extract::extract_text(filepath)?;

    let text = text.replace('\u{00A0}', " ");

    if text.len() == 0 {
        return Ok(());
    }

    let embeddings = get_embedding(&text)?;

    //This has to be added to the db with the file name as the primary key.
    let avg_embeddings = average_embedding(&embeddings);

    add_embedding(pool, filename, filepath, &avg_embeddings, cur_dir).await?;

    Ok(())
}

*/

/*
pub async fn extract_pdf(
    cur_dir: &str,
    filename: &str,
    filepath: &str,
    pool: &SqlitePool,
) -> Result<(), Box<dyn std::error::Error>> {
    //Extracting the text contents from the PDF
    let text = pdf_extract::extract_text(filepath)?;

    let text = text.replace('\u{00A0}', " ");

    if text.is_empty() {
        return Ok(());
    }

    let embeddings = get_embedding(&text)?;

    if embeddings.is_empty() {
        return Ok(());
    }

    //This has to be added to the db with the file name as the primary key.
    let avg_embeddings = average_embedding(&embeddings);
    add_embedding(pool, filename, filepath, &avg_embeddings, cur_dir).await?;

    Ok(())
}

*/

pub async fn extract_pdf(
    cur_dir: &str,
    filename: &str,
    filepath: &str,
    pool: &SqlitePool,
) -> Result<(), Box<dyn std::error::Error>> {
    //Extracting the text contents from the PDF
    let mut doc = match PdfDocument::open(filepath) {
        Ok(p) => p,

        Err(e) => {
            println!("Error: {}", e);
            return Ok(());
        }
    };

    let mut text = String::new();

    let mut page_count = doc.page_count().unwrap();

    if page_count > 50 {
        page_count = 50;
    }

    println!("Current File: {}", filename);

    for i in 0..page_count {
        let content = match doc.extract_text(i) {
            Ok(t) => t,

            Err(_) => {
                println!("Error");
                return Ok(());
            }
        };

        text.push_str(&content);
        text.push_str("\n\n");
    }

    //Text is extracted

    process(&text, filename, filepath, cur_dir, pool).await?;
    Ok(())
}
