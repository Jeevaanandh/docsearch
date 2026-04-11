use std::process::Command;

pub fn open(filename: &str) {
    Command::new("open")
        .arg(filename)
        .spawn()
        .expect("Failed to open file")
        .wait();
}
