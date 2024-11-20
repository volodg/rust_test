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
        self.head.take().map(|old_head| {
            if let Some(next) = old_head.borrow_mut().next.take() {
                next.borrow_mut().prev = Weak::new();
                self.head = Some(next);
            } else {
                self.tail = None;
            }
            self.length -= 1;
            Rc::try_unwrap(old_head).ok().unwrap().into_inner().value
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
