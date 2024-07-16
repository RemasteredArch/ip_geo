#![allow(dead_code)]

use celes::Country;
use core::fmt;
use either::Either;
use serde::{
    de::{Unexpected, Visitor},
    Deserialize, Deserializer,
};
use std::{
    cmp::Ordering,
    error::Error,
    fmt::Display,
    fs,
    net::{Ipv4Addr, Ipv6Addr},
    ops::RangeInclusive,
    path::Path,
    str::FromStr,
};

#[derive(Debug)]
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

pub type Ipv4AddrEntry<T> = IpAddrEntry<Ipv4Addr, T>;
pub type Ipv6AddrEntry<T> = IpAddrEntry<Ipv6Addr, T>;

/// Example usage:
///
/// ```
/// let entry = Ipv4AddrEntry::new(
///     Ipv4Addr::new(1, 0, 0, 0),
///     Ipv4Addr::new(2, 2, 2, 2),
///     "contents",
/// )
/// .unwrap();
///
/// assert!(entry < Ipv4Addr::new(0, 0, 0, 0));
/// assert!(entry == Ipv4Addr::new(1, 1, 1, 1));
/// assert!(entry > Ipv4Addr::new(3, 3, 3, 3));
/// ```
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

pub fn parse_ipv6_file(path: Box<Path>, len: usize) -> IpAddrMap<Ipv6Addr, Country> {
    #[derive(Deserialize, Debug)]
    struct Schema {
        #[serde(deserialize_with = "deserialize_ipv6")]
        start: Ipv6Addr,

        #[serde(deserialize_with = "deserialize_ipv6")]
        end: Ipv6Addr,

        #[serde(with = "either::serde_untagged")]
        country: Either<Country, String>,
    }

    let file = fs::File::open(&path)
        .unwrap_or_else(|_| panic!("Could not open IPv6 database at {}", path.to_string_lossy()));
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .comment(Some(b'#'))
        .from_reader(file);

    let mut map = IpAddrMap::new_with_capacity(len);

    for entry in reader.deserialize() {
        let data: Schema = entry.unwrap();

        let country = match data.country {
            Either::Left(country) => country,
            Either::Right(unrecognized) => {
                eprintln!("Unrecognized country or region '{unrecognized}'!");
                continue;
            }
        };

        map.insert(Ipv6AddrEntry::new(data.start, data.end, country).unwrap());
    }

    map.cleanup();

    map
}

fn deserialize_ipv6<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Ipv6Addr, D::Error> {
    pub struct Ipv6Deserializer;

    impl<'de> Visitor<'de> for Ipv6Deserializer {
        type Value = Ipv6Addr;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "an IPv6 address")
        }

        fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            const MASK: u128 = 0xFF;

            let array = std::array::from_fn(|index| {
                let shift = (7 - index) * u16::BITS as usize;

                (v >> shift & MASK) as u16
            });

            Ok(array.into())
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ipv6Addr::from_str(v).map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
        }
    }

    deserializer.deserialize_str(Ipv6Deserializer)
}

pub fn parse_ipv4_file(path: Box<Path>, len: usize) -> IpAddrMap<Ipv4Addr, Country> {
    #[derive(Deserialize, Debug)]
    struct Schema {
        #[serde(deserialize_with = "deserialize_ipv4")]
        start: Ipv4Addr,

        #[serde(deserialize_with = "deserialize_ipv4")]
        end: Ipv4Addr,

        #[serde(with = "either::serde_untagged")]
        country: Either<Country, String>,
    }

    let file = fs::File::open(&path)
        .unwrap_or_else(|_| panic!("Could not open IPv4 database at {}", path.to_string_lossy()));
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .comment(Some(b'#'))
        .from_reader(file);

    let mut map = IpAddrMap::new_with_capacity(len);

    for entry in reader.deserialize() {
        let data: Schema = entry.unwrap();

        let country = match data.country {
            Either::Left(country) => country,
            Either::Right(unrecognized) => {
                eprintln!("Unrecognized country or region '{unrecognized}'!");
                continue;
            }
        };

        map.insert(Ipv4AddrEntry::new(data.start, data.end, country).unwrap());
    }

    map.cleanup();

    map
}

fn deserialize_ipv4<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Ipv4Addr, D::Error> {
    pub struct Ipv4Deserializer;

    impl<'de> Visitor<'de> for Ipv4Deserializer {
        type Value = Ipv4Addr;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "an IPv4 address")
        }

        fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            const MASK: u32 = 0xFF;

            let array = std::array::from_fn(|index| {
                let shift = (3 - index) * u8::BITS as usize;

                (v >> shift & MASK) as u8
            });

            Ok(array.into())
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ipv4Addr::from_str(v).map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
        }
    }

    deserializer.deserialize_u32(Ipv4Deserializer)
}
