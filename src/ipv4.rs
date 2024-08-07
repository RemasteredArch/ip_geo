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

use crate::{
    country_list::{get_countries, Country},
    IpAddrEntry, IpAddrMap,
};
use serde::{
    de::{Unexpected, Visitor},
    Deserialize, Deserializer,
};
use std::{fs, net::Ipv4Addr, path::Path, str::FromStr};

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
/// `comment` is used internally as a `u8` by taking the last byte of `comment` (`comment as u8`).
///
/// Example usage:
///
/// ```rust
/// use std::{
///     io::Write,
///     net::Ipv4Addr,
///     str::FromStr,
/// };
///
/// let start_a = Ipv4Addr::new(1, 1, 1, 1);
/// let end_a = Ipv4Addr::new(3, 3, 3, 3);
/// let value_a = "BE".into();
/// let middle_a = Ipv4Addr::new(2, 2, 2, 2);
///
/// let start_b = Ipv4Addr::new(4, 4, 4, 4);
/// let end_b = Ipv4Addr::new(6, 6, 6, 6);
/// let value_b = "CA".into();
/// let middle_b = Ipv4Addr::new(5, 5, 5, 5);
///
/// let mut temp_file = tempfile::NamedTempFile::new().unwrap();
/// write!(
///     temp_file,
///     "{},{},{value_a}\n{},{},{value_b}\n",
///     u32::from(start_a),
///     u32::from(end_a),
///     u32::from(start_b),
///     u32::from(end_b),
/// )
/// .unwrap();
/// let path = temp_file.path().into();
/// let len = 2;
///
/// let mut ipv4_map = ip_geo::ipv4::parse_ipv4_file(path, len, Some('#'));
///
/// assert_eq!(ipv4_map.search(middle_a).unwrap().code, value_a);
/// assert_eq!(ipv4_map.search(middle_b).unwrap().code, value_b);
///
/// assert_eq!(ipv4_map.get_from_index_as_ref(0).unwrap().value().code, value_a);
/// assert_eq!(ipv4_map.get_from_index_as_ref(1).unwrap().value().code, value_b);
/// ```
pub fn parse_ipv4_file(
    path: Box<Path>,
    len: usize,
    comment: Option<char>,
) -> IpAddrMap<Ipv4Addr, Country> {
    #[derive(Deserialize, Debug)]
    struct Schema {
        #[serde(deserialize_with = "deserialize_ipv4")]
        start: Ipv4Addr,

        #[serde(deserialize_with = "deserialize_ipv4")]
        end: Ipv4Addr,

        country_code: Box<str>,
    }

    let file = fs::File::open(&path)
        .unwrap_or_else(|_| panic!("Could not open IPv4 database at {}", path.to_string_lossy()));
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .comment(comment.map(|c| c as u8))
        .from_reader(file);

    let mut map = IpAddrMap::new_with_capacity(len);
    let countries = get_countries();

    for entry in reader.deserialize() {
        let data: Schema = entry.unwrap();

        let code = data.country_code.as_ref();

        // Ensure that it is a recognized country
        match countries.get(code).cloned() {
            Some(country) => {
                // Only add ranges with associated countries
                if country.code != "??".into() {
                    map.insert(Ipv4AddrEntry::new(data.start, data.end, country).unwrap());
                }
            }
            None => eprintln!("Unrecognized country or region '{}'!", data.country_code),
        }
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
            Ok(Ipv4Addr::from_bits(v))
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
