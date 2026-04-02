use std::{
    fs::read_dir,
    thread
};

use crate::pdf_test::extract_pdf;


pub fn parse_directory(){
    let mut handles = Vec::new();

    for entry_res in read_dir(".").unwrap() {
        let entry = entry_res.unwrap();
        let file_name_buf = entry.file_name();
        let file_name = file_name_buf.to_str().unwrap();

        if !file_name.starts_with(".") && entry.file_type().unwrap().is_file() && file_name.ends_with(".pdf") {
            let path_str= entry.path().to_string_lossy().to_string();

            let handle= thread::spawn(move || {

                
                match extract_pdf(&path_str){
                    Ok(_) => {
                        println!("{} was a Success!", path_str);
                    }

                    Err(_) => {
                        println!("{} Failed", path_str);
                    }

                

                }

            });

            handles.push(handle);


            
        }
    }

    for i in handles{
        i.join().unwrap();
    }
}