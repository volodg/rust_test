use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};

pub mod doubly_linked_list;
pub mod fixed_hash_table;

fn main() -> io::Result<()> {
    let file_path = "./resources/98-0.txt"; // Замените на путь к вашему файлу
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?; // Читаем строку из файла
        let words: Vec<&str> = line.split_whitespace().collect(); // Разделяем строку на слова
        println!("{:?}", words); // Печатаем слова
    }

    Ok(())
}
