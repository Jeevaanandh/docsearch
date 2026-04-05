use std::fs::read_dir;

use sqlx::SqlitePool;
use tokio::task;

use crate::pdf_test::extract_pdf;
use crate::repository::db::get_paths;

pub async fn check_diff(pool: &SqlitePool) {
    let files = match get_paths(pool).await {
        Ok(paths) => paths,

        Err(e) => {
            println!("Error in getting paths from the DB: {}", e);
            return;
        }
    };

    for entry_res in read_dir(".").unwrap() {
        let entry = entry_res.unwrap();
        let file_name_buf = entry.file_name();
        let file_name = file_name_buf.to_str().unwrap().to_string();

        if !files.contains(&file_name) {
            match extract_pdf(&file_name, pool).await {
                Ok(_) => {
                    println!("{} added Successfully!!", file_name);
                }

                Err(_) => {
                    println!("Error in adding {} from diff", file_name);
                    return;
                }
            };
        }
    }
}

pub async fn parse_directory(pool: &SqlitePool) {
    let mut handles = Vec::new();

    for entry_res in read_dir(".").unwrap() {
        let entry = entry_res.unwrap();
        let file_name_buf = entry.file_name();
        let file_name = file_name_buf.to_str().unwrap();

        if !file_name.starts_with(".")
            && entry.file_type().unwrap().is_file()
            && file_name.ends_with(".pdf")
        {
            let path_str = entry.path().to_string_lossy().to_string();
            let pool = pool.clone();

            let handle = task::spawn(async move {
                match extract_pdf(&path_str, &pool).await {
                    Ok(_) => {
                        println!("{} was a Success!", path_str);
                    }

                    Err(e) => {
                        println!("{} Failed {}", path_str, e);
                    }
                }
            });

            handles.push(handle);
        }
    }

    for i in handles {
        i.await.unwrap();
    }
}
