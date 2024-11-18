use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
enum Slot<K: Clone, V: Clone> {
    Empty,
    Occupied(K, V),
    Deleted,
}

pub struct FixedHashTable<K: Clone, V: Clone> {
    table: Vec<Slot<K, V>>,
    size: usize,
    count: usize,
}

impl<K: Eq + Hash + Clone, V: Clone> FixedHashTable<K, V> {
    pub fn new(size: usize) -> Self {
        Self {
            table: vec![Slot::Empty; size],
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
                    self.table[index] = Slot::Occupied(key, value); // Обновляем значение
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
                Slot::Empty => return None, // Ключа точно нет
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
                Slot::Empty => return Err("Key not found"), // Ключа точно нет
                _ => index = (index + 1) % self.size,
            }
        }

        Err("Key not found")
    }
}

fn main() {
    let mut hash_table = FixedHashTable::new(5);

    // Вставка
    hash_table.insert("apple", 5).unwrap();
    hash_table.insert("banana", 10).unwrap();
    hash_table.insert("orange", 15).unwrap();

    // Получение
    println!("{:?}", hash_table.get(&"apple"));  // Some(5)
    println!("{:?}", hash_table.get(&"banana")); // Some(10)
    println!("{:?}", hash_table.get(&"grape"));  // None

    // Удаление
    hash_table.delete(&"banana").unwrap();
    println!("{:?}", hash_table.get(&"banana")); // None

    // Ошибка при переполнении
    hash_table.insert("grape", 20).unwrap();
    hash_table.insert("pear", 25).unwrap();
    match hash_table.insert("melon", 30) {
        Ok(_) => println!("Inserted"),
        Err(e) => println!("Error: {}", e), // Error: Hash table is full
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;

    #[test]
    fn it_works() {
        let hash_map: HashMap<String, String> = HashMap::new();
        println!("{:?}", hash_map); // None

        let mut hash_table = FixedHashTable::new(10);

        // Вставка элементов
        hash_table.insert("apple", 5);
        hash_table.insert("banana", 10);
        hash_table.insert("orange", 15);

        // Получение значений
        println!("{:?}", hash_table.get(&"apple"));  // Some(5)
        println!("{:?}", hash_table.get(&"banana")); // Some(10)
        println!("{:?}", hash_table.get(&"grape"));  // None

        // Удаление элемента
        hash_table.delete(&"banana");
        println!("{:?}", hash_table.get(&"banana")); // None
    }
}
