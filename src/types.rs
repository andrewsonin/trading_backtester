use std::{convert::Infallible, fmt::{Debug, Display}, hash::Hash, marker::PhantomData};

pub use chrono::{
    Duration,
    NaiveDate as Date,
    NaiveDateTime as DateTime,
    NaiveTime as Time,
    Timelike,
};

pub trait Id: Hash + Ord + Copy + Send + Sync + Display + Debug {}

impl<T: Hash + Ord + Copy + Send + Sync + Display + Debug> Id for T {}

pub trait Named<Name: Id> {
    fn get_name(&self) -> Name;
}

pub trait TimeSync {
    fn current_datetime_mut(&mut self) -> &mut DateTime;
}

pub trait Agent {
    type Action;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NeverType<K>((Infallible, PhantomData<K>));

pub type Nothing = NeverType<()>;
