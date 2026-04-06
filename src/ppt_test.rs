use quick_xml::{Reader, events::Event};

use std::{
    fs::File,
    io::{BufReader, Read},
};

use zip::ZipArchive;

use crate::pdf_test::{average_embedding, get_embedding};

use sqlx::SqlitePool;

use crate::repository::db::add_embedding;

fn read_pptx(filename: &str) -> Result<String, Box<dyn std::error::Error>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader)?;

    let mut pdf_text = String::from("");

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            let text = extract_text_from_xml(&contents)?;

            pdf_text.push_str(&text);
            pdf_text.push_str("\n\n");
        }
    }

    Ok(pdf_text)
}

fn extract_text_from_xml(xml: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut txt = String::new();
    let mut buf = Vec::new();
    let mut in_text = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                // <a:t> tags contain the actual text content
                if e.name().as_ref() == b"a:t" {
                    in_text = true;
                }
            }
            Ok(Event::Text(e)) if in_text => {
                txt.push_str(&e.unescape()?.into_owned());
                txt.push(' ');
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"a:t" {
                    in_text = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Box::new(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(txt.trim().to_string())
}

pub async fn parse_ppt(
    filename: &str,
    pool: &SqlitePool,
) -> Result<(), Box<dyn std::error::Error>> {
    let text = read_pptx(filename)?;

    let embeddings = get_embedding(&text)?;

    let avg_embeddings = average_embedding(&embeddings);

    println!("{}: {}", filename, text);

    add_embedding(pool, filename, &avg_embeddings).await?;

    Ok(())
}
