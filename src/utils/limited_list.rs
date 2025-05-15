use std::collections::VecDeque;
use std::fmt::Debug;

pub struct LimitedList<T>
    where
        T: Eq + Clone + Debug {
    pub list: VecDeque<T>,
    capacity: usize,
}

impl<T> LimitedList<T> where
    T: Eq + Clone + Debug, {
    pub fn new(capacity: usize) -> Self {
        Self {
            list: VecDeque::new(),
            capacity,
        }
    }

    pub fn add(&mut self, item: T) {
        if self.list.len() == self.capacity {
            self.list.pop_front();
        }
        self.list.push_back(item);
    }
    pub fn all(&self) -> Vec<&T> {
        self.list.iter().collect()
    }
    pub fn del(&mut self, item: &T) {
        let position = self.list.iter().position(|x| x == item);
        if let Some(element) = position {
            self.list.remove(element);
        }
    }
    pub fn is_full(&self) -> bool {
        self.list.len() == self.capacity
    }
}