use std::fs::read_dir;

use sqlx::SqlitePool;
use tokio::task;

use crate::pdf_test::extract_pdf;
use crate::repository::db::{delete_path, get_paths};

use crate::ppt_test::parse_ppt;

pub async fn check_diff(pool: &SqlitePool) {
    let files = match get_paths(pool).await {
        Ok(paths) => paths,

        Err(e) => {
            println!("Error in getting paths from the DB: {}", e);
            return;
        }
    };

    let mut cur_files: Vec<String> = Vec::new();

    let mut handles = Vec::new();
    for entry_res in read_dir(".").unwrap() {
        let entry = entry_res.unwrap();
        let file_name_buf = entry.file_name();
        let file_name = file_name_buf.to_str().unwrap().to_string();

        let file_clone = file_name.clone();

        if !file_name.starts_with(".")
            && entry.file_type().unwrap().is_file()
            && ((file_name.ends_with(".pdf")) || file_name.ends_with(".pptx"))
        {
            cur_files.push(file_clone);
        }

        if !files.contains(&file_name) {
            if file_name.ends_with(".pptx") {
                let p = pool.clone();
                let f = file_name.clone();

                let handle = task::spawn(async move {
                    match parse_ppt(&f, &p).await {
                        Ok(_) => {
                            println!("{} was added Succseefully", f);
                        }

                        Err(_) => {
                            println!("Error in adding: {}", f);
                        }
                    }
                });

                handles.push(handle);

                continue;
            }
            if !file_name.ends_with(".pdf") {
                continue;
            }

            let p = pool.clone();
            let f = file_name.clone();

            let handle = task::spawn(async move {
                match extract_pdf(&f, &p).await {
                    Ok(_) => {
                        println!("{} was added Successfully!", f);
                    }

                    Err(_) => {
                        println!("Error in adding {}", f);
                    }
                }
            });

            handles.push(handle);
        }
    }

    for i in handles {
        i.await.unwrap();
    }

    check_deletions(&cur_files, pool).await;
}

async fn check_deletions(cur_files: &Vec<String>, pool: &SqlitePool) {
    let db_paths = match get_paths(pool).await {
        Ok(paths) => paths,

        Err(e) => {
            println!("Error in getting db paths (diff): {}", e);
            return;
        }
    };

    for i in db_paths {
        if !cur_files.contains(&i) {
            match delete_path(&i, pool).await {
                Ok(_) => {
                    println!("{} was deleted Successfully!", i);
                }

                Err(e) => {
                    println!("Delete Failed for: {}, Reason: {}", i, e);
                }
            };
        }
    }
}

//Tried doing something, found out it was worse than before

async fn process_file(files: &Vec<String>, pool: &SqlitePool) {
    let mut handles = Vec::new();
    for file in files {
        let f = file.clone();
        let p = pool.clone();

        if file.ends_with(".pdf") {
            let handle = task::spawn(async move {
                match extract_pdf(&f, &p).await {
                    Ok(_) => {
                        println!("{} was a Success!", f);
                    }

                    Err(e) => {
                        println!("{} Failed {}", f, e);
                    }
                }
            });

            handles.push(handle);
        } else {
            let handle = task::spawn(async move {
                match parse_ppt(&f, &p).await {
                    Ok(_) => {
                        println!("{} was a Success!", f);
                    }

                    Err(e) => {
                        println!("{} Failed {}", f, e);
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

pub async fn parse_directory2(pool: &SqlitePool) {
    let mut files: Vec<String> = Vec::new();
    let mut handles = Vec::new();

    for entry_res in read_dir(".").unwrap() {
        let entry = entry_res.unwrap();
        let file_name_buf = entry.file_name();
        let file_name = file_name_buf.to_str().unwrap();

        if !file_name.starts_with(".")
            && entry.file_type().unwrap().is_file()
            && ((file_name.ends_with(".pdf")) || file_name.ends_with(".pptx"))
        {
            files.push(file_name.to_string());

            if files.len() == 10 {
                let p = pool.clone();
                let fls = files.clone();

                let handle = task::spawn(async move {
                    process_file(&fls, &p).await;
                });

                handles.push(handle);

                files = Vec::new();
            }
        }
    }

    if !files.is_empty() {
        let p = pool.clone();
        let fls = files.clone();

        let handle = task::spawn(async move {
            process_file(&fls, &p).await;
        });

        handles.push(handle);
    }

    for i in handles {
        i.await.unwrap();
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
            && ((file_name.ends_with(".pdf")) || file_name.ends_with(".pptx"))
        {
            let path_str = entry.path().to_string_lossy().to_string();
            let pool = pool.clone();

            if file_name.ends_with(".pptx") {
                let handle = task::spawn(async move {
                    match parse_ppt(&path_str, &pool).await {
                        Ok(_) => {
                            println!("{} was a Success!", path_str);
                        }

                        Err(e) => {
                            println!("{} failed. Error: {}", path_str, e);
                        }
                    }
                });

                handles.push(handle);

                continue;
            }

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
