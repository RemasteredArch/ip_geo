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

use clap::Parser;
use ip_geo::{country::Country, IpAddrMap};
use std::net::{Ipv4Addr, Ipv6Addr};

use warp::Filter;

mod arguments;
use arguments::Arguments;

#[tokio::main]
pub async fn main() {
    let arguments = arguments::get_config(Arguments::parse());

    // Safety: `arguments::get_config()` implements default values
    let port = arguments.port.unwrap();

    let mut ipv4_map = parse_ipv4(&arguments);
    let mut ipv6_map = parse_ipv6(&arguments);

    let ipv4 = warp::path!("ipv4" / Ipv4Addr)
        .map(|ipv4_addr: Ipv4Addr| search_ipv4(ipv4_addr, &mut ipv4_map));

    println!("Serving on http://127.0.0.1:{port}/");

    let routes = warp::get().and(warp::path("v0")).and(ipv4);

    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

fn search_ipv4(ipv4_addr: Ipv4Addr, ipv4_map: &mut IpAddrMap<Ipv4Addr, Country>) -> String {
    ipv4_map.search(ipv4_addr).unwrap().name.to_string()
}

/// Lossily converts a char to a byte.
///
/// Where a char is multiple bytes, it returns only the first byte.
fn char_to_byte(char: char) -> u8 {
    // How could this be improved?
    char.to_string().as_bytes().first().unwrap().to_owned()
}

/// For a given set of arguments, parse and return the IPv4 database into an `IpAddrMap`.
fn parse_ipv4(arguments: &Arguments) -> IpAddrMap<Ipv4Addr, Country> {
    // Safety: `arguments::get_config()` implements default values
    let ipv4_path = arguments.ipv4_path.clone().unwrap();
    let ipv4_len = arguments.ipv4_len.unwrap();
    let ipv4_comment = arguments.ipv4_comment.map(char_to_byte);

    ip_geo::ipv4::parse_ipv4_file(ipv4_path, ipv4_len, ipv4_comment)
}

/// For a given set of arguments, parse and return the IPv6 database into an `IpAddrMap`.
fn parse_ipv6(arguments: &Arguments) -> IpAddrMap<Ipv6Addr, Country> {
    // Safety: `arguments::get_config()` implements default values
    let ipv6_path = arguments.ipv6_path.clone().unwrap();
    let ipv6_len = arguments.ipv6_len.unwrap();
    let ipv6_comment = arguments.ipv6_comment.map(char_to_byte);

    ip_geo::ipv6::parse_ipv6_file(ipv6_path, ipv6_len, ipv6_comment)
}
