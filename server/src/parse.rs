// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright © 2024 RemasteredArch
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

use std::{
    net::{Ipv4Addr, Ipv6Addr},
    sync::Arc,
};

use ip_geo::{country_list::Country, IpAddrMap};

use crate::arguments::Arguments;

/// For a given set of `Arguments`, parse the specified IPv4 and IPv6 databases into `IpAddrMap`s
/// and return them in a struct holding them as `Arc`s.
pub fn parse_ip_maps(arguments: &Arguments) -> Maps {
    Maps::new(parse_ipv4(arguments), parse_ipv6(arguments))
}

/// A simple struct for passing around `IpAddrMaps`.
pub struct Maps {
    pub v4: Arc<IpAddrMap<Ipv4Addr, Country>>,
    pub v6: Arc<IpAddrMap<Ipv6Addr, Country>>,
}

impl Maps {
    /// Create a new `Maps` from IPv4 and IPv6 `IpAddrMap`s.
    pub fn new(
        ipv4_map: IpAddrMap<Ipv4Addr, Country>,
        ipv6_map: IpAddrMap<Ipv6Addr, Country>,
    ) -> Self {
        Self {
            v4: Arc::new(ipv4_map),
            v6: Arc::new(ipv6_map),
        }
    }
}

/// For a given set of arguments, parse and return the IPv4 database into a clean `IpAddrMap`.
fn parse_ipv4(arguments: &Arguments) -> IpAddrMap<Ipv4Addr, Country> {
    // Safety: `arguments::get_config()` implements default values
    let path = arguments.ipv4_db_path.clone().unwrap();
    let file_length = arguments.ipv4_db_len.unwrap();
    let comment = arguments.ipv4_db_comment;

    let mut map = ip_geo::ipv4::parse_ipv4_file(path, file_length, comment);
    map.cleanup();

    map
}

/// For a given set of arguments, parse and return the IPv6 database into an `IpAddrMap`.
fn parse_ipv6(arguments: &Arguments) -> IpAddrMap<Ipv6Addr, Country> {
    // Safety: `arguments::get_config()` implements default values
    let path = arguments.ipv6_db_path.clone().unwrap();
    let file_length = arguments.ipv6_db_len.unwrap();
    let comment = arguments.ipv6_db_comment;

    let mut map = ip_geo::ipv6::parse_ipv6_file(path, file_length, comment);
    map.cleanup();

    map
}
