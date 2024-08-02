// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright © 2024 RemasteredArch
// Copyright © 2024 Jaxydog
//
// This file is part of ip_geo.
//
// ip_geo is free software: you can redistribute it and/or modify it under the terms of the GNU
// Affero General Public License as published by the Free Software Foundation, either version 3 of
// the License, or (at your option) any later version.
//
// ip_geo is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
// even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with ip_geo. If
// not, see <https://www.gnu.org/licenses/>.

use core::fmt;
use std::{cmp::Ordering, error::Error, fmt::Display, ops::RangeInclusive};

pub mod country;
pub mod country_list;
pub mod ipv4;
pub mod ipv6;

/// Stores a searchable list of `IpAddrEntries`.
///
/// Example usage:
///
/// ```rust
/// use std::net::Ipv4Addr;
/// use ip_geo::{IpAddrEntry, IpAddrMap};
///
/// let entry_a = IpAddrEntry::new(
///     Ipv4Addr::new(1, 1, 1, 1),
///     Ipv4Addr::new(3, 3, 3, 3),
///     "a",
/// )
/// .unwrap();
///
/// let entry_b = IpAddrEntry::new(
///     Ipv4Addr::new(4, 4, 4, 4),
///     Ipv4Addr::new(6, 6, 6, 6),
///     "b",
/// )
/// .unwrap();
///
/// let mut map = IpAddrMap::new();
/// map.insert(entry_a.clone());
/// map.insert(entry_b.clone());
///
/// assert_eq!(map.search(Ipv4Addr::new(2, 2, 2, 2)), Some("a").as_ref());
/// assert_eq!(map.search(Ipv4Addr::new(5, 5, 5, 5)), Some("b").as_ref());
///
/// assert_eq!(map.get_from_index_as_ref(0), Some(&entry_a));
/// assert_eq!(map.get_from_index_as_ref(1), Some(&entry_b));
/// ```
#[derive(Debug)]
pub struct IpAddrMap<A: Ord + Copy, T: PartialEq> {
    inner: Vec<IpAddrEntry<A, T>>,
    dirty: bool,
}

impl<A: Ord + Copy, T: PartialEq> IpAddrMap<A, T> {
    /// Create a new, unsized instance of `Self`.
    pub const fn new() -> Self {
        Self {
            inner: vec![],
            dirty: false,
        }
    }

    /// Create a new instance of `Self` with a starting capacity for the internal `Vec`.
    pub fn new_with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            dirty: false,
        }
    }

    /// Add another entry into the map.
    pub fn insert(&mut self, entry: IpAddrEntry<A, T>) {
        self.inner.push(entry);
        self.dirty = true;
    }

    /// For a given IP address, find the value of the stored entries the contains it, else `None`.
    ///
    /// Cleans the map first, if necessary.
    pub fn search(&mut self, address: A) -> Option<&T> {
        // Cleans the map, making `search_unsafe()` safe to use.
        self.cleanup();

        self.search_unsafe(address)
    }

    /// For a given IP address, find the value of the stored entries the contains it, else `None`.
    ///
    /// Panics if called on a dirty map. Only call this if you don't have a mutable reference and
    /// are *certain* that it has been cleaned first.
    pub fn search_unsafe(&self, address: A) -> Option<&T> {
        if self.dirty {
            panic!("Tried to search dirty IPAddrMap");
        }

        let index = self
            .inner
            .binary_search_by(|e| e.partial_cmp(&address).unwrap())
            .ok()?;

        Some(self.inner[index].value())
    }

    /// If necessary, prepare internal `Vec` for searching by performing a dedup, sort, and shrink.
    ///
    /// This is called by `Self::search()`, it should not be necessary to perform manually unless
    /// it is used in an interactive program and you want to do as much work as possible before interactivity.
    pub fn cleanup(&mut self) {
        if !self.dirty {
            return;
        }

        self.inner.dedup_by(|a, b| a == b);
        self.inner.sort_unstable_by_key(|e| (e.start, e.end));
        self.inner.shrink_to_fit(); // Assumes that you will only ever cleanup after you're done
                                    // adding to the map.
        self.dirty = false;
    }

    /// Return the entry at a given index in the internal `Vec` as a reference.
    pub fn get_from_index_as_ref(&self, index: usize) -> Option<&IpAddrEntry<A, T>> {
        self.inner.get(index)
    }

    /// Return the length of the internal `Vec`.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the internal `Vec` is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<A: Ord + Copy, T: PartialEq> Default for IpAddrMap<A, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Ord + Copy, T: PartialEq> IntoIterator for IpAddrMap<A, T> {
    type Item = IpAddrEntry<A, T>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

/// Stores a range of IP addresses and a value.
///
/// Example usage:
///
/// ```rust
/// use std::net::Ipv4Addr;
/// use ip_geo::IpAddrEntry;
///
/// let entry = IpAddrEntry::new(
///     Ipv4Addr::new(1, 1, 1, 1),
///     Ipv4Addr::new(3, 3, 3, 3),
///     "contents",
/// )
/// .unwrap();
///
/// assert!(entry > Ipv4Addr::new(0, 0, 0, 0));
/// assert!(entry == Ipv4Addr::new(2, 2, 2, 2));
/// assert!(entry < Ipv4Addr::new(4, 4, 4, 4));
/// ```
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct IpAddrEntry<A: Ord + Copy, T> {
    start: A,
    end: A,
    value: T,
}

impl<A: Ord + Copy, T> IpAddrEntry<A, T> {
    /// Create a new instance of self.
    /// Takes the start and end of an IP address range and a corresponding value.
    ///
    /// Will error if given an invalid range.
    pub fn new(start: A, end: A, value: T) -> Result<Self, EmptyRangeError> {
        if start <= end {
            Ok(Self { start, end, value })
        } else {
            Err(EmptyRangeError)
        }
    }

    /// Return a reference to the first value of the stored IP address range.
    pub const fn start(&self) -> &A {
        &self.start
    }

    /// Return a mutable reference to the first value of the stored IP address range.
    pub fn start_mut(&mut self) -> &mut A {
        &mut self.start
    }

    /// Return a reference to the last value of the stored IP address range.
    pub const fn end(&self) -> &A {
        &self.end
    }

    /// Return a mutable reference to the last value of the stored IP address range.
    pub fn end_mut(&mut self) -> &mut A {
        &mut self.end
    }

    /// Return a reference to the stored value.
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Return a mutable reference to the stored value.
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Return the stored IP address range as a range: `(start..=end)`
    pub const fn range(&self) -> RangeInclusive<A> {
        self.start..=self.end
    }

    /// Return a tuple of the start of the stored IP address range, the end of the stored IP
    /// address range, and the stored value: `(start, end, value)`
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
            v if v > &self.end => Some(Ordering::Less),
            v if v < &self.start => Some(Ordering::Greater),
            v if self == v => Some(Ordering::Equal),
            _ => unreachable!(),
        }
    }
}

/// The error returned when attemping to construct an invalid range.
#[derive(Debug)]
pub struct EmptyRangeError;

impl Error for EmptyRangeError {}

impl Display for EmptyRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the given range is empty")
    }
}
