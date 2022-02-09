use std::{cmp::Reverse, collections::BinaryHeap};

#[derive(Default)]
/// A priority queue implemented with a binary heap.
///
/// This will be a min-heap.
pub struct LessElementBinaryHeap<T: Ord>(pub BinaryHeap<Reverse<T>>);

impl<T: Ord> LessElementBinaryHeap<T>
{
    /// Removes the lowest item from the binary heap and returns it, or None if it is empty.
    pub fn pop(&mut self) -> Option<T> {
        if let Some(Reverse(message)) = self.0.pop() {
            Some(message)
        } else {
            None
        }
    }

    /// Pushes an item onto the binary heap.
    pub fn push(&mut self, item: T) {
        self.0.push(Reverse(item))
    }

    /// Returns the length of the binary heap.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Ord> Extend<T> for LessElementBinaryHeap<T>
{
    fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I) {
        self.0.extend(iter.into_iter().map(Reverse))
    }
}

/// Structure to provide push-only access for the inner [`LessElementBinaryHeap`].
pub struct MessageReceiver<'a, T: Ord> (&'a mut LessElementBinaryHeap<T>);

impl<'a, T: Ord> MessageReceiver<'a, T> {
    /// Creates a new instance of the [`MessageReceiver`].
    pub fn new(queue: &'a mut LessElementBinaryHeap<T>) -> Self {
        Self(queue)
    }

    /// Pushes an item onto the binary heap.
    pub fn push(&mut self, item: T) {
        self.0.push(item)
    }
}

impl<'a, T: Ord> Extend<T> for MessageReceiver<'a, T> {
    fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}