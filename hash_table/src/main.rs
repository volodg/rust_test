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

    for line in reader.lines() {
        let line = line?;
        let words: Vec<&str> = line.split_whitespace().collect();

        for word in words {
            if data_set.get(word).is_none() && data_set.len() == max_size {
                let key = data_set.get_first().expect("can not be empty").0;
                data_set.delete(key.as_str());
            }

            // TODO: add get_mut method
        }
    }

    Ok(())
}
