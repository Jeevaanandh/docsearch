use pdf_extract;


pub fn extract_pdf(file: &str){
    let text = pdf_extract::extract_text(file).unwrap();
    println!("{}", text);

    let chunks: Vec<String>= text.split("\n\n")  // double newline = paragraph
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

}