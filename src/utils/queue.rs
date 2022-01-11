use std::{cmp::Reverse, collections::BinaryHeap};

#[derive(Default)]
pub struct LessElementBinaryHeap<T: Ord>(pub BinaryHeap<Reverse<T>>);

impl<T: Ord> LessElementBinaryHeap<T>
{
    pub(crate) fn pop(&mut self) -> Option<T> {
        if let Some(Reverse(message)) = self.0.pop() {
            Some(message)
        } else {
            None
        }
    }

    pub(crate) fn push(&mut self, item: T) {
        self.0.push(Reverse(item))
    }
}

impl<T: Ord> Extend<T> for LessElementBinaryHeap<T>
{
    fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I)
    {
        self.0.extend(iter.into_iter().map(Reverse))
    }
}