use crate::embed::{average_embedding, get_embedding};
use crate::repository::db::search_db;
use dirs;
use sqlx::SqlitePool;
use std::{error::Error, result::Result};
use usearch::{new_index, Index, IndexOptions, MetricKind, ScalarKind};

fn faiss_impl(
    embeddings: &Vec<Vec<f32>>,
    prompt_embeddings: &Vec<f32>,
) -> Result<(Vec<u64>, Vec<f32>), Box<dyn Error>> {
    let len = embeddings.len();
    let dim = embeddings[0].len();
    let n_res = len.min(10);

    let options = IndexOptions {
        dimensions: dim,               // necessary for most metric kinds
        metric: MetricKind::L2sq,      // or ::L2sq, ::Cos ...
        quantization: ScalarKind::F32, // or ::F32, ::F16, ::E5M2, ::E4M3, ::E3M2, ::E2M3, ::U8, ::I8, ::B1x8 ...
        connectivity: 0,               // zero for auto
        expansion_add: 0,              // zero for auto
        expansion_search: 0,
        multi: false,
    };

    let mut index: Index = new_index(&options)?;
    index.reserve(len)?;

    for (i, emb) in embeddings.iter().enumerate() {
        index.add(i as u64, emb)?;
    }

    let matches = index.search(prompt_embeddings, n_res)?;

    Ok((matches.keys, matches.distances))
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

    let keys = faiss_result.0;
    let dist = faiss_result.1;

    let mut result_files: Vec<String> = Vec::new();
    let mut result_paths: Vec<String> = Vec::new();

    for i in 0..keys.len() {
        let ind = keys[i] as usize;

        result_files.push(files[ind].clone());
        result_paths.push(filepaths[ind].clone());
    }

    (result_files, result_paths)
}
