use crate::doubly_linked_list;
use crate::doubly_linked_list::DoublyLinkedList;
use proptest::proptest;
use std::cell::{Ref, RefCell};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

// TODO:
// 1. random hasher
// 3. Cleanup Deleted
// 4. Avoid Clone
// 5. Move FixedHashTable to separate module
// 6. add rehash

enum Slot<K, V> {
    Empty,
    Occupied(K, V, Rc<RefCell<doubly_linked_list::Node<K>>>),
    Deleted,
}

pub struct FixedHashTable<K, V> {
    table: Vec<Slot<K, V>>,
    history: DoublyLinkedList<K>,
    size: usize,
    count: usize,
}

// TODO don't clone key
impl<K: Eq + Hash + Clone, V> FixedHashTable<K, V> {
    pub fn new(size: usize) -> Self {
        let mut table = Vec::with_capacity(size);
        for _ in 0..size {
            table.push(Slot::Empty)
        }
        Self {
            table,
            history: DoublyLinkedList::<K>::new(),
            size,
            count: 0,
        }
    }

    // TODO, use random hasher to avoid hash attacks
    fn hash(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize % self.size
    }

    // TODO try to return replaced value
    pub fn insert(&mut self, key: K, value: V) -> bool {
        if self.count >= self.size {
            return false;
        }

        let mut index = self.hash(&key);
        for _ in 0..self.size {
            match &self.table[index] {
                Slot::Empty | Slot::Deleted => {
                    let node = self.history.push_back(key.clone());

                    self.table[index] = Slot::Occupied(key, value, node);
                    self.count += 1;
                    return true;
                }
                Slot::Occupied(existing_key, _, prev_node) if existing_key == &key => {
                    self.history.remove(prev_node.clone());
                    let node = self.history.push_back(key.clone());

                    self.table[index] = Slot::Occupied(key, value, node);
                    return true;
                }
                _ => index = (index + 1) % self.size,
            }
        }

        panic!("can not insert, invalid state")
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut index = self.hash(key);
        for _ in 0..self.size {
            match &self.table[index] {
                Slot::Occupied(existing_key, value, _) if existing_key == key => {
                    return Some(value)
                }
                Slot::Empty => return None,
                _ => index = (index + 1) % self.size,
            }
        }

        None
    }

    // TODO: return deleted value
    pub fn delete(&mut self, key: &K) -> bool {
        let mut index = self.hash(key);
        for _ in 0..self.size {
            match &self.table[index] {
                Slot::Occupied(existing_key, _, node) if existing_key == key => {
                    self.history.remove(node.clone());

                    self.table[index] = Slot::Deleted;
                    self.count -= 1;
                    return true;
                }
                Slot::Empty => return false,
                _ => index = (index + 1) % self.size,
            }
        }

        false
    }

    pub fn get_last(&self) -> Option<(Ref<K>, &V)> {
        match self.history.back() {
            Some(key) => {
                let value = self.get(&key).expect("Key should exist in the hash table");
                Some((key, value))
            },
            None => None,
        }
    }

    pub fn get_first(&self) -> Option<(Ref<K>, &V)> {
        match self.history.front() {
            Some(key) => {
                let value = self.get(&key).expect("Key should exist in the hash table");
                Some((key, value))
            },
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_get_delete() {
        let mut hash_table = FixedHashTable::new(10);

        assert!(hash_table.insert("apple", 5));
        assert!(hash_table.insert("banana", 10));
        assert!(hash_table.insert("orange", 15));

        assert_eq!(hash_table.get(&"apple"), Some(5).as_ref());
        assert_eq!(hash_table.get(&"banana"), Some(10).as_ref());
        assert_eq!(hash_table.get(&"grape"), None);

        assert!(hash_table.delete(&"banana"));
        assert_eq!(hash_table.get(&"banana"), None);
    }


    #[test]
    fn test_get_first_get_last() {
        let mut hash_table = FixedHashTable::new(10);

        assert!(hash_table.insert("1", 1));
        assert!(hash_table.insert("2", 2));
        assert!(hash_table.insert("2", 22));
        assert!(hash_table.insert("3", 3));
        assert!(hash_table.insert("3", 33));
        assert!(hash_table.insert("4", 4));

        hash_table.delete(&"1");
        hash_table.delete(&"4");

        let Some((key, value)) = hash_table.get_first() else {
            panic!("existing key value expected")
        };
        assert_eq!(*key, "2");
        assert_eq!(*value, 22);

        let Some((key, value)) = hash_table.get_last() else {
            panic!("existing key value expected")
        };
        assert_eq!(*key, "3");
        assert_eq!(*value, 33);
    }
}

proptest! {
    #[test]
    fn test_insert_delete_insert_get_equivalence(keys in proptest::collection::vec(proptest::string::string_regex("[a-z]{1,5}").unwrap(), 1..20), values in proptest::collection::vec(0..100, 1..20)) {
        use std::collections::HashMap;
        let mut table = FixedHashTable::new(20);
        let mut map = HashMap::new();

        for (key, value) in keys.iter().zip(values.iter()) {
            assert!(table.insert(key.clone(), *value));
            map.insert(key.clone(), *value);
        }

        for (key, _) in keys.iter().zip(values.iter()) {
            assert_eq!(table.delete(key), map.remove(key).is_some());
        }

        for (key, value) in keys.iter().zip(values.iter()) {
            assert!(table.insert(key.clone(), *value));
            map.insert(key.clone(), *value);
        }

        for (key, _) in keys.iter().zip(values.iter()) {
            assert_eq!(table.get(key), map.get(key));
        }
    }

    #[test]
    fn test_size_limit(keys in proptest::collection::vec(proptest::string::string_regex("[a-z]{1,5}").unwrap(), 1..20)) {
        use std::collections::HashMap;

        let max_size = 5;

        let mut table = FixedHashTable::new(max_size); // Ограничиваем размер до 5
        let mut map = HashMap::new();

        for (i, key) in keys.iter().enumerate() {
            if map.len() < max_size {
                assert!(table.insert(key.clone(), i));
                map.insert(key.clone(), i);
            } else {
                assert!(!table.insert(key.clone(), i));
            }
        }

        for (key, &value) in map.iter() {
            assert_eq!(table.get(key), Some(&value));
        }
    }
}
