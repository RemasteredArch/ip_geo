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

use crate::country::Country;
use crate::country_list::get_countries;
use crate::{IpAddrEntry, IpAddrMap};
use serde::de::Unexpected;
use serde::{de::Visitor, Deserialize, Deserializer};
use std::str::FromStr;
use std::{fs, net::Ipv6Addr, path::Path};

/// Stores a range of IPv6 addresses and a value.
///
/// Example usage:
///
/// ```rust
/// use std::net::Ipv6Addr;
/// use ip_geo::ipv6::Ipv6AddrEntry;
///
/// let entry = Ipv6AddrEntry::new(
///     Ipv6Addr::new(1, 1, 1, 1, 1, 1, 1, 1),
///     Ipv6Addr::new(3, 3, 3, 3, 3, 3, 3, 3),
///     "contents",
/// )
/// .unwrap();
///
/// assert!(entry > Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0));
/// assert!(entry == Ipv6Addr::new(2, 2, 2, 2, 2, 2, 2, 2));
/// assert!(entry < Ipv6Addr::new(4, 4, 4, 4, 4, 4, 4, 4));
/// ```
pub type Ipv6AddrEntry<T> = IpAddrEntry<Ipv6Addr, T>;

/// For given IPv6 database file of a given length, parse it into an `IpAddrMap` holding IPv6 addresses.
///
/// `comment` is used internally as a `u8` by taking the last byte of `comment` (`comment as u8`).
///
/// Example usage:
///
/// ```rust
/// use std::{
///     io::Write,
///     net::Ipv6Addr,
///     str::FromStr,
/// };
///
/// let start_a = "1::";
/// let end_a = "3::";
/// let value_a = "BE".into();
/// let middle_a = Ipv6Addr::from_str("2::").unwrap();
///
/// let start_b = "4::";
/// let end_b = "6::";
/// let value_b = "CA".into();
/// let middle_b = Ipv6Addr::from_str("5::").unwrap();
///
/// let mut temp_file = tempfile::NamedTempFile::new().unwrap();
/// write!(
///     temp_file,
///     "{},{},{value_a}\n{},{},{value_b}\n",
///     start_a, end_a, start_b, end_b,
/// )
/// .unwrap();
/// let path = temp_file.path().into();
/// let len = 2;
///
/// let mut ipv6_map = ip_geo::ipv6::parse_ipv6_file(path, len, Some('#'));
///
/// assert_eq!(ipv6_map.search(middle_a).unwrap().code, value_a);
/// assert_eq!(ipv6_map.search(middle_b).unwrap().code, value_b);
///
/// assert_eq!(ipv6_map.get_from_index_as_ref(0).unwrap().value().code, value_a);
/// assert_eq!(ipv6_map.get_from_index_as_ref(1).unwrap().value().code, value_b);
/// ```
pub fn parse_ipv6_file(
    path: Box<Path>,
    len: usize,
    comment: Option<char>,
) -> IpAddrMap<Ipv6Addr, Country> {
    #[derive(Deserialize, Debug)]
    struct Schema {
        #[serde(deserialize_with = "deserialize_ipv6")]
        start: Ipv6Addr,

        #[serde(deserialize_with = "deserialize_ipv6")]
        end: Ipv6Addr,

        country_code: String,
    }

    let file = fs::File::open(&path)
        .unwrap_or_else(|_| panic!("Could not open IPv6 database at {}", path.to_string_lossy()));
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .comment(comment.map(|c| c as u8))
        .from_reader(file);

    let mut map = IpAddrMap::new_with_capacity(len);
    let countries = get_countries();

    for entry in reader.deserialize() {
        let data: Schema = entry.unwrap();

        if let Some(country) = Country::from_code(&data.country_code, &countries) {
            // If not an unrecognized IP block,
            if country.code != "??".into() {
                map.insert(Ipv6AddrEntry::new(data.start, data.end, country).unwrap());
            }
        } else {
            eprintln!("Unrecognized country or region '{}'!", data.country_code);
        }
    }

    map.cleanup();

    map
}

/// Serde deserializer to convert a `u128` into an `Ipv6Addr`.
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
            Ok(Ipv6Addr::from_bits(v))
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
