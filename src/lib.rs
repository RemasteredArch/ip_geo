#![allow(dead_code)]

use core::fmt;
use std::{
    cmp::Ordering,
    error::Error,
    fmt::Display,
    net::{Ipv4Addr, Ipv6Addr},
    ops::RangeInclusive,
};

pub struct IpAddrMap<A: Ord + Copy, T: PartialEq> {
    inner: Vec<IpAddrEntry<A, T>>,
    dirty: bool,
}

impl<A: Ord + Copy, T: PartialEq> IpAddrMap<A, T> {
    pub const fn new() -> Self {
        Self {
            inner: vec![],
            dirty: false,
        }
    }

    pub fn new_with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            dirty: false,
        }
    }

    pub fn insert(&mut self, entry: IpAddrEntry<A, T>) {
        self.inner.push(entry);
        self.dirty = true;
    }

    pub fn search(&mut self, address: A) -> Option<&T> {
        self.cleanup();

        self.inner
            .binary_search_by(|e| e.partial_cmp(&address).unwrap())
            .ok()
            .map(|i| self.inner[i].value())
    }

    pub fn cleanup(&mut self) {
        if self.dirty {
            self.inner.dedup_by(|a, b| a == b);
            self.inner.sort_unstable_by_key(|e| (e.start, e.end));
            self.inner.shrink_to_fit(); // Assumes that you will only ever cleanup after you're done
                                        // adding to the map
            self.dirty = false;
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<A: Ord + Copy, T: PartialEq> IntoIterator for IpAddrMap<A, T> {
    type Item = IpAddrEntry<A, T>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

pub type Ipv4AddrEntry<T> = IpAddrEntry<Ipv4Addr, T>;
pub type Ipv6AddrEntry<T> = IpAddrEntry<Ipv6Addr, T>;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct IpAddrEntry<A: Ord + Copy, T> {
    start: A,
    end: A,
    value: T,
}

impl<A: Ord + Copy, T> IpAddrEntry<A, T> {
    pub fn new(start: A, end: A, value: T) -> Result<Self, EmptyRangeError> {
        if start <= end {
            Ok(Self { start, end, value })
        } else {
            Err(EmptyRangeError)
        }
    }

    pub const fn start(&self) -> &A {
        &self.start
    }

    pub fn start_mut(&mut self) -> &mut A {
        &mut self.start
    }

    pub const fn end(&self) -> &A {
        &self.end
    }

    pub fn end_mut(&mut self) -> &mut A {
        &mut self.end
    }

    pub const fn value(&self) -> &T {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub const fn range(&self) -> RangeInclusive<A> {
        self.start..=self.end
    }

    pub fn unwrap(self) -> (A, A, T) {
        let Self { start, end, value } = self;

        (start, end, value)
    }
}

impl<A: Ord + Copy, T> PartialEq<A> for IpAddrEntry<A, T> {
    fn eq(&self, other: &A) -> bool {
        self.range().contains(other)
    }
}

impl<A: Ord + Copy, T> PartialOrd<A> for IpAddrEntry<A, T> {
    fn partial_cmp(&self, other: &A) -> Option<std::cmp::Ordering> {
        match other {
            v if v < &self.start => Some(Ordering::Less),
            v if v > &self.end => Some(Ordering::Greater),
            v if self == v => Some(Ordering::Equal),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct EmptyRangeError;

impl Error for EmptyRangeError {}

impl Display for EmptyRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the given range is empty")
    }
}
