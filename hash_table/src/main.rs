use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use crate::fixed_hash_table::FixedHashTable;

pub mod doubly_linked_list;
pub mod fixed_hash_table;

fn main() -> io::Result<()> {
    let file_path = "./hash_table/resources/98-0.txt";
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let max_size = 100;
    let mut data_set = FixedHashTable::<String, usize>::new(100);

    let mut data_set2 = HashMap::<String, usize>::new();
    data_set2.insert("1".to_string(), 2);
    data_set2.insert("3".to_string(), 3);
    println!("test: {:?}", data_set2);

    for line in reader.lines() {
        let line = line?;
        let words: Vec<&str> = line.split_whitespace().collect();

        for word in words {
            if data_set.get(word).is_none() && data_set.len() == max_size {
                let key = data_set.get_first().expect("can not be empty").0;
                data_set.delete(key.as_str());
            }

            // TODO: map.entry("key".to_string()).or_insert(0);
            if let Some(count) = data_set.get_mut(word) {
                *count += 1;
            } else {
                _ = data_set.insert(word.to_string(), 1);
            };
        }
    }

    println!("test: {:?}", data_set);

    Ok(())
}
