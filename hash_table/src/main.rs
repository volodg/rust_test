use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use crate::fixed_hash_table::FixedHashTable;

pub mod doubly_linked_list;
pub mod fixed_hash_table;

// TODO
// 1. Decouple doubly_linked_list from fixed_hash_table

fn main() -> io::Result<()> {
    let file_path = "./hash_table/resources/98-0.txt";
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let max_size = 50;
    let mut data_set = FixedHashTable::<String, usize>::new(max_size);

    for line in reader.lines() {
        let line = line?;
        let words: Vec<&str> = line.split_whitespace().collect();

        for word in words {
            let word = word.trim_matches(|x: char| x.is_ascii_punctuation()).to_lowercase();

            if data_set.get(&word).is_none() && data_set.len() == max_size {
                let key = data_set.get_first().expect("can not be empty").0;
                data_set.delete(key.as_str());
            }

            // TODO: map.entry("key".to_string()).or_insert(0);
            if let Some(count) = data_set.get_mut(&word) {
                *count += 1;
            } else {
                _ = data_set.insert(word, 1);
            };
        }
    }

    println!("data_set: {:?}", data_set);

    Ok(())
}
