
use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};

pub struct Node<T> {
    value: T,
    prev: Weak<RefCell<Node<T>>>,
    next: Option<Rc<RefCell<Node<T>>>>,
}

pub struct DoublyLinkedList<T> {
    head: Option<Rc<RefCell<Node<T>>>>,
    tail: Option<Rc<RefCell<Node<T>>>>,
    length: usize,
}

// TODO
// 1. re-review all methods
// 2. add unit tests
// 3. delete not used methods - #[allow(dead_code)]
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
                // Если список пуст
                self.head = Some(new_node.clone());
                self.tail = Some(new_node.clone());
            }
        }

        self.length += 1;
        new_node
    }

    // TODO review
    #[allow(dead_code)]
    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.take().map(|old_tail| {
            if let Some(prev) = old_tail.borrow_mut().prev.upgrade() {
                prev.borrow_mut().next = None;
                self.tail = Some(prev);
            } else {
                // Если список стал пустым
                self.head = None;
            }
            self.length -= 1;
            Rc::try_unwrap(old_tail).ok().unwrap().into_inner().value
        })
    }

    // TODO review
    #[allow(dead_code)]
    pub fn push_front(&mut self, value: T) -> Rc<RefCell<Node<T>>> {
        let new_node = Rc::new(RefCell::new(Node {
            value,
            prev: Weak::new(),
            next: None,
        }));

        match self.head.take() {
            Some(old_head) => {
                old_head.borrow_mut().prev = Rc::downgrade(&new_node);
                new_node.borrow_mut().next = Some(old_head);
                self.head = Some(new_node.clone());
            }
            None => {
                // Если список пуст
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
                // Если список стал пустым
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
    #[allow(dead_code)]
    pub fn front(&self) -> Option<Ref<T>> {
        self.head.as_ref().map(|node| Ref::map(node.borrow(), |n| &n.value))
    }

    // TODO review
    #[allow(dead_code)]
    pub fn back(&self) -> Option<Ref<T>> {
        self.tail.as_ref().map(|node| Ref::map(node.borrow(), |n| &n.value))
    }

    // TODO review
    #[allow(dead_code)]
    pub fn front_mut(&self) -> Option<RefMut<T>> {
        self.head.as_ref().map(|node| RefMut::map(node.borrow_mut(), |n| &mut n.value))
    }

    // TODO review
    #[allow(dead_code)]
    pub fn back_mut(&self) -> Option<RefMut<T>> {
        self.tail.as_ref().map(|node| RefMut::map(node.borrow_mut(), |n| &mut n.value))
    }

    // TODO review
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.length
    }

    // TODO review
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

impl<T> Drop for DoublyLinkedList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}
