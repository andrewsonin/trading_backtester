use std::{convert::Infallible, fmt::{Debug, Display}, hash::Hash, marker::PhantomData};

pub use chrono::{
    Duration,
    NaiveDate as Date,
    NaiveDateTime as DateTime,
    NaiveTime as Time,
    Timelike,
};

/// Markers and being automatically derived for types, which can be names and keys.
pub trait Id: Hash + Ord + Copy + Send + Sync + Display + Debug {}

impl<T: Hash + Ord + Copy + Send + Sync + Display + Debug> Id for T {}

/// Supposed to be implemented for named entities to get their IDs.
pub trait Named<Name: Id> {
    /// Retrieves name of the entity.
    fn get_name(&self) -> Name;
}

/// Allows entities to be reported about current global time.
pub trait TimeSync {
    /// Return reference to the `DateTime` of the current entity.
    fn current_datetime_mut(&mut self) -> &mut DateTime;
}

/// Markers agents (i.e. [traders](crate::interface::trader),
/// [brokers](crate::interface::broker) and [exchanges](crate::interface::exchange)).
pub trait Agent {
    type Action;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
/// Parametrized type to marker code branches that will never happen.
/// Can be used as a [message type](crate::interface::message)
/// to indicate that the given message bridge between agents is not used.
pub struct NeverType<K>((Infallible, PhantomData<K>));

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
/// Non-parametrized type to marker code branches that will never happen.
/// Can be used as a [message type](crate::interface::message)
/// to indicate that the given message bridge between agents is not used.
pub struct Nothing(Infallible);
