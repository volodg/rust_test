use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use proptest::proptest;

// TODO:
// 0. unit tests
// 1. random hasher
// 2. better error codes
// #[derive(Clone, Debug)]
enum Slot<K, V> {
    Empty,
    Occupied(K, V),
    Deleted,
}

pub struct FixedHashTable<K, V> {
    table: Vec<Slot<K, V>>,
    size: usize,
    count: usize,
}

impl<K: Eq + Hash, V> FixedHashTable<K, V> {
    pub fn new(size: usize) -> Self {
        let mut table = Vec::with_capacity(size);
        for _ in 0..size {
            table.push(Slot::Empty)
        }
        Self {
            table,
            size,
            count: 0,
        }
    }

    fn hash(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize % self.size
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<(), &'static str> {
        if self.count >= self.size {
            return Err("Hash table is full");
        }

        let mut index = self.hash(&key);
        for _ in 0..self.size {
            match &self.table[index] {
                Slot::Empty | Slot::Deleted => {
                    self.table[index] = Slot::Occupied(key, value);
                    self.count += 1;
                    return Ok(());
                }
                Slot::Occupied(existing_key, _) if existing_key == &key => {
                    self.table[index] = Slot::Occupied(key, value); // Update value
                    return Ok(());
                }
                _ => index = (index + 1) % self.size,
            }
        }

        Err("No available slots found")
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut index = self.hash(key);
        for _ in 0..self.size {
            match &self.table[index] {
                Slot::Occupied(existing_key, value) if existing_key == key => return Some(value),
                Slot::Empty => return None,
                _ => index = (index + 1) % self.size,
            }
        }

        None
    }

    pub fn delete(&mut self, key: &K) -> Result<(), &'static str> {
        let mut index = self.hash(key);
        for _ in 0..self.size {
            match &self.table[index] {
                Slot::Occupied(existing_key, _) if existing_key == key => {
                    self.table[index] = Slot::Deleted;
                    self.count -= 1;
                    return Ok(());
                }
                Slot::Empty => return Err("Key not found"),
                _ => index = (index + 1) % self.size,
            }
        }

        Err("Key not found")
    }
}

#[cfg(test)]
mod tests {
    // use std::collections::HashMap;
    use super::*;

    #[test]
    fn test_insert_get_delete() {
        // let mut hash_map = HashMap::new();
        // hash_map.insert(Key {}, Value {});

        let mut hash_table = FixedHashTable::new(10);

        assert!(hash_table.insert("apple", 5).is_ok());
        assert!(hash_table.insert("banana", 10).is_ok());
        assert!(hash_table.insert("orange", 15).is_ok());

        assert_eq!(hash_table.get(&"apple"), Some(5).as_ref());
        assert_eq!(hash_table.get(&"banana"), Some(10).as_ref());
        assert_eq!(hash_table.get(&"grape"), None);

        assert!(hash_table.delete(&"banana").is_ok());
        assert_eq!(hash_table.get(&"banana"), None);
    }
}

proptest! {
    #[test]
    fn test_insert_get_equivalence(keys in proptest::collection::vec(proptest::string::string_regex("[a-z]{1,5}").unwrap(), 1..10), values in proptest::collection::vec(0..100, 1..10)) {
        let mut table = FixedHashTable::new(20);
        let mut map = HashMap::new();

        for (key, value) in keys.iter().zip(values.iter()) {
            table.insert(key.clone(), *value).unwrap();
            map.insert(key.clone(), *value);
        }

        for (key, _) in keys.iter().zip(values.iter()) {
            assert_eq!(table.get(key), map.get(key));
        }
    }

    // #[test]
    // fn test_insert_delete_equivalence(keys in proptest::collection::vec(proptest::string::string_regex("[a-z]{1,5}").unwrap(), 1..10), values in proptest::collection::vec(0..100, 1..10)) {
    //     let mut table = FixedHashTable::new(20);
    //     let mut map = HashMap::new();
    //
    //     // Вставляем данные в обе структуры
    //     for (key, value) in keys.iter().zip(values.iter()) {
    //         table.insert(key.clone(), *value).unwrap();
    //         map.insert(key.clone(), *value);
    //     }
    //
    //     for (key, _) in keys.iter().zip(values.iter()) {
    //         table.delete(key).unwrap();
    //         map.remove(key);
    //     }
    //
    //     for (key, _) in keys.iter().zip(values.iter()) {
    //         assert_eq!(table.get(key), map.get(key));
    //     }
    // }

    // #[test]
    // fn test_size_limit(keys in proptest::collection::vec(proptest::string::string_regex("[a-z]{1,5}").unwrap(), 1..10)) {
    //     let mut table = FixedHashTable::new(5); // Ограничиваем размер до 5
    //     let mut map = HashMap::new();
    //
    //     for (i, key) in keys.iter().enumerate() {
    //         if i < 5 {
    //             table.insert(key.clone(), i).unwrap();
    //             map.insert(key.clone(), i);
    //         } else {
    //             assert_eq!(table.insert(key.clone(), i), Err("Hash table is full"));
    //         }
    //     }
    //
    //     for (key, &value) in map.iter() {
    //         assert_eq!(table.get(key), Some(&value));
    //     }
    // }
}
