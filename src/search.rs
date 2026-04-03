use crate::pdf_test::{average_embedding, get_embedding};
use crate::repository::db::search_db;
use dirs;
use fastembed::{EmbeddingModel, Error, InitOptions, TextEmbedding};
use sqlx::SqlitePool;
use std::result::Result;

use faiss::{Index, MetricType, index::SearchResult, index_factory};

fn faiss_impl(
    embeddings: &Vec<Vec<f32>>,
    prompt_embedding: &Vec<f32>,
) -> Result<SearchResult<f32>, faiss::error::Error> {
    let dim = embeddings[0].len();
    let mut index = index_factory(dim as u32, "Flat", MetricType::L2)?;

    let flat: Vec<f32> = embeddings.iter().flatten().cloned().collect();

    index.add(&flat)?;

    let result = index.search(prompt_embedding, 1)?;

    Ok(result)
}

pub async fn search(prompt: &str, pool: &SqlitePool) {
    let (paths, embeddings) = match search_db(pool).await {
        Ok(res) => res,

        Err(e) => {
            println!("Error in DB search: {}", e);
            return;
        }
    };

    let prompt_embedding = match get_embedding(prompt) {
        Ok(emb) => emb,

        Err(_) => {
            println!("Error in getting the embedding");
            return;
        }
    };

    let avg_emb_prompt = average_embedding(&prompt_embedding);

    let faiss_result = match faiss_impl(&embeddings, &avg_emb_prompt) {
        Ok(res) => res,

        Err(_) => {
            println!("Error in Faiss");
            return;
        }
    };

    let best_index = faiss_result.labels[0].to_native() as usize;

    println!("Best match index: {}", best_index);
    println!("Best Match: {}", paths[best_index]);
}
