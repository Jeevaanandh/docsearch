use crate::embed::{average_embedding, get_embedding};
use crate::repository::db::search_db;
use dirs;
use sqlx::SqlitePool;
use std::result::Result;

use faiss::{index::SearchResult, index_factory, Index, MetricType};

fn faiss_impl(
    embeddings: &Vec<Vec<f32>>,
    prompt_embedding: &Vec<f32>,
) -> Result<SearchResult<f32>, faiss::error::Error> {
    let len = embeddings.len();
    let n_res = len.min(10);
    let dim = embeddings[0].len();
    let mut index = index_factory(dim as u32, "Flat", MetricType::L2)?;

    let flat: Vec<f32> = embeddings.iter().flatten().cloned().collect();

    index.add(&flat)?;

    let result = index.search(prompt_embedding, n_res)?;

    Ok(result)
}

pub async fn search(prompt: &str, pool: &SqlitePool) -> (Vec<String>, Vec<String>) {
    let (files, embeddings, filepaths) = match search_db(pool).await {
        Ok(res) => res,

        Err(e) => {
            println!("Error in DB search: {}", e);
            return (Vec::new(), Vec::new());
        }
    };

    let prompt_embedding = match get_embedding(prompt) {
        Ok(emb) => emb,

        Err(_) => {
            println!("Error in getting the embedding");
            return (Vec::new(), Vec::new());
        }
    };

    let avg_emb_prompt = average_embedding(&prompt_embedding);

    let faiss_result = match faiss_impl(&embeddings, &avg_emb_prompt) {
        Ok(res) => res,

        Err(e) => {
            println!("Error in Faiss: {}", e);
            return (Vec::new(), Vec::new());
        }
    };

    let mut counter = 0;
    let mut result_files: Vec<String> = Vec::new();
    let mut result_paths: Vec<String> = Vec::new();

    for i in faiss_result.labels {
        if faiss_result.distances[counter] >= 1.0 {
            counter += 1;
            continue;
        }
        let ind = i.to_native() as usize;
        result_files.push(files[ind].clone());
        result_paths.push(filepaths[ind].clone());

        counter += 1;
    }

    (result_files, result_paths)
}
