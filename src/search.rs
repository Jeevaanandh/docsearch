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
    let len = embeddings.len();
    let n_res = len % 10;
    let dim = embeddings[0].len();
    let mut index = index_factory(dim as u32, "Flat", MetricType::L2)?;

    let flat: Vec<f32> = embeddings.iter().flatten().cloned().collect();

    index.add(&flat)?;

    let result = index.search(prompt_embedding, n_res)?;

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

    let mut counter = 0;
    println!("----------TOP RESULTS----------");
    for i in faiss_result.labels {
        let ind = i.to_native() as usize;
        println!(
            "{}) {}, Distance: {:.2}",
            counter + 1,
            paths[ind],
            faiss_result.distances[counter]
        );
        counter += 1;
    }
}
