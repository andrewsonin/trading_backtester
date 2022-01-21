use std::{cmp::Reverse, collections::BinaryHeap};

#[derive(Default)]
pub struct LessElementBinaryHeap<T: Ord>(pub BinaryHeap<Reverse<T>>);

impl<T: Ord> LessElementBinaryHeap<T>
{
    pub fn pop(&mut self) -> Option<T> {
        if let Some(Reverse(message)) = self.0.pop() {
            Some(message)
        } else {
            None
        }
    }

    pub fn push(&mut self, item: T) {
        self.0.push(Reverse(item))
    }
}

impl<T: Ord> Extend<T> for LessElementBinaryHeap<T>
{
    fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I) {
        self.0.extend(iter.into_iter().map(Reverse))
    }
}

pub struct MessageReceiver<'a, T: Ord> (&'a mut LessElementBinaryHeap<T>);

impl<'a, T: Ord> MessageReceiver<'a, T> {
    pub fn new(queue: &'a mut LessElementBinaryHeap<T>) -> Self {
        Self(queue)
    }

    pub fn push(&mut self, item: T) {
        self.0.push(item)
    }
}

impl<'a, T: Ord> Extend<T> for MessageReceiver<'a, T> {
    fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}