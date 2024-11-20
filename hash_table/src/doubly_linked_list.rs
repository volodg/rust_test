use std::cell::{Ref, RefCell};
use std::rc::{Rc, Weak};

pub struct Node<T> {
    value: T,
    prev: Weak<RefCell<Node<T>>>,
    next: Option<Rc<RefCell<Node<T>>>>,
}

#[derive(Default)]
pub struct DoublyLinkedList<T> {
    head: Option<Rc<RefCell<Node<T>>>>,
    tail: Option<Rc<RefCell<Node<T>>>>,
    length: usize,
}

// TODO
// 1. re-review all methods
// 2. add unit tests
impl<T> DoublyLinkedList<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            length: 0,
        }
    }

    // TODO review
    pub fn push_back(&mut self, value: T) -> Rc<RefCell<Node<T>>> {
        let new_node = Rc::new(RefCell::new(Node {
            value,
            prev: Weak::new(),
            next: None,
        }));

        match self.tail.take() {
            Some(old_tail) => {
                old_tail.borrow_mut().next = Some(new_node.clone());
                new_node.borrow_mut().prev = Rc::downgrade(&old_tail);
                self.tail = Some(new_node.clone());
            }
            None => {
                self.head = Some(new_node.clone());
                self.tail = Some(new_node.clone());
            }
        }

        self.length += 1;
        new_node
    }

    // TODO review
    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().and_then(|old_head| {
            if let Some(next) = old_head.borrow_mut().next.take() {
                next.borrow_mut().prev = Weak::new();
                self.head = Some(next);
            } else {
                self.tail = None;
            }
            self.length -= 1;
            let node = Rc::try_unwrap(old_head).ok()?.into_inner();
            Some(node.value)
        })
    }

    pub fn remove(&mut self, node: Rc<RefCell<Node<T>>>) {
        let mut node_ref = node.borrow_mut();

        if let Some(prev) = node_ref.prev.upgrade() {
            prev.borrow_mut().next = node_ref.next.clone();
        } else {
            self.head = node_ref.next.clone();
        }

        if let Some(next) = node_ref.next.take() {
            next.borrow_mut().prev = node_ref.prev.clone();
        } else {
            self.tail = node_ref.prev.upgrade();
        }

        node_ref.next = None;
        node_ref.prev = Default::default();

        self.length -= 1;
    }

    // TODO review
    pub fn front(&self) -> Option<Ref<T>> {
        self.head
            .as_ref()
            .map(|node| Ref::map(node.borrow(), |n| &n.value))
    }

    // TODO review
    pub fn back(&self) -> Option<Ref<T>> {
        self.tail
            .as_ref()
            .map(|node| Ref::map(node.borrow(), |n| &n.value))
    }
}

impl<T> Drop for DoublyLinkedList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_back_and_back() {
        let mut list = DoublyLinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        assert_eq!(*list.back().unwrap(), 3);
        assert_eq!(list.length, 3);
    }

    #[test]
    fn test_pop_front() {
        let mut list = DoublyLinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.length, 0);
    }

    #[test]
    fn test_remove_node() {
        let mut list = DoublyLinkedList::new();
        {
            let node1 = list.push_back(1);
            let node2 = list.push_back(2);
            let node3 = list.push_back(3);

            // Remove the middle node
            list.remove(node2);
        }
        assert_eq!(list.length, 2);

        // Check remaining nodes
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn test_front_and_back() {
        let mut list = DoublyLinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        assert_eq!(*list.front().unwrap(), 1);
        assert_eq!(*list.back().unwrap(), 3);
    }

    #[test]
    fn test_empty_list() {
        let list: DoublyLinkedList<i32> = DoublyLinkedList::new();
        assert!(list.front().is_none());
        assert!(list.back().is_none());
        assert_eq!(list.length, 0);
    }

    #[test]
    fn test_remove_head() {
        let mut list = DoublyLinkedList::new();
        let node1 = list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        list.remove(node1);
        assert_eq!(list.length, 2);

        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn test_remove_tail() {
        let mut list = DoublyLinkedList::new();
        list.push_back(1);
        list.push_back(2);
        let node3 = list.push_back(3);

        list.remove(node3);
        assert_eq!(list.length, 2);

        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn test_remove_single_node() {
        let mut list = DoublyLinkedList::new();
        let node1 = list.push_back(1);

        list.remove(node1);
        assert_eq!(list.length, 0);

        assert!(list.front().is_none());
        assert!(list.back().is_none());
    }
}
