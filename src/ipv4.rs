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

use crate::{IpAddrEntry, IpAddrMap};
use celes::Country;
use either::Either;
use serde::de::Unexpected;
use serde::{de::Visitor, Deserialize, Deserializer};
use std::str::FromStr;
use std::{fs, net::Ipv4Addr, path::Path};

/// Stores a range of IPv4 addresses and a value.
///
/// Example usage:
///
/// ```rust
/// use std::net::Ipv4Addr;
/// use ip_geo::ipv4::Ipv4AddrEntry;
///
/// let entry = Ipv4AddrEntry::new(
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
pub type Ipv4AddrEntry<T> = IpAddrEntry<Ipv4Addr, T>;

/// For given IPv4 database file of a given length, parse it into an `IpAddrMap` holding IPv4 addresses.
///
/// Example usage:
///
/// ```rust
/// use std::{
///     fs::{File},
///     io::{Read, Write},
///     net::Ipv4Addr,
///     str::FromStr,
/// };
/// use ip_geo::{parse_ipv4_file, Ipv4AddrEntry};
///
/// let entry_a = (
///     Ipv4Addr::new(1, 1, 1, 1),
///     Ipv4Addr::new(3, 3, 3, 3),
///     "BE",
/// );
/// let raw_start_a: u32  = entry_a.0.into();
/// let raw_end_a: u32  = entry_a.1.into();
/// let middle_a = Ipv4Addr::new(2, 2, 2, 2);
///
/// let entry_b = (
///     Ipv4Addr::new(4, 4, 4, 4),
///     Ipv4Addr::new(6, 6, 6, 6),
///     "CA",
/// );
/// let raw_start_b: u32  = entry_b.0.into();
/// let raw_end_b: u32  = entry_b.1.into();
/// let middle_b = Ipv4Addr::new(5, 5, 5, 5);
///
/// let mut temp_file = tempfile::NamedTempFile::new().unwrap();
/// write!(
///     temp_file,
///     "{},{},{}\n{},{},{}\n",
///     raw_start_a, raw_end_a, entry_a.2, raw_start_b, raw_end_b, entry_b.2,
/// )
/// .unwrap();
/// let path = temp_file.path().into();
/// let len = 200_000;
///
/// let mut ipv4_map = parse_ipv4_file(path, len);
///
/// assert_eq!(ipv4_map.search(middle_a).unwrap().alpha2, entry_a.2);
/// assert_eq!(ipv4_map.search(middle_b).unwrap().alpha2, entry_b.2);
///
/// assert_eq!(ipv4_map.get_from_index_as_ref(0).unwrap().value().alpha2, entry_a.2);
/// assert_eq!(ipv4_map.get_from_index_as_ref(1).unwrap().value().alpha2, entry_b.2);
/// ```
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

/// Serde deserializer to convert a `u32` into an `Ipv4Addr`.
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
