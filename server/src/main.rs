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
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    sync::Arc,
};

use warp::Filter;

mod arguments;
use arguments::Arguments;

#[tokio::main]
pub async fn main() {
    let arguments = arguments::get_config(Arguments::parse());

    // Safety: `arguments::get_config()` implements default values
    let port = arguments.port.unwrap();

    let ipv4_map = Arc::new(parse_ipv4(&arguments));
    let ipv6_map = Arc::new(parse_ipv6(&arguments));

    let search_ipv4 = move |ipv4_addr: Ipv4Addr| search_clean_ip_map(ipv4_addr, &ipv4_map);
    let search_ipv6 = move |ipv6_addr: Ipv6Addr| search_clean_ip_map(ipv6_addr, &ipv6_map);

    let ipv4 = warp::path!("ipv4" / Ipv4Addr).map(search_ipv4);
    let ipv6 = warp::path!("ipv6" / Ipv6Addr).map(search_ipv6);

    println!("Serving on http://127.0.0.1:{port}/");

    let routes = warp::get().and(warp::path("v0")).and(ipv4.or(ipv6));

    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

/// Search an IPv4 address map for an IP address.
///
/// Assumes that the `IpAddrMap` is clean, otherwise it will panic.
fn search_clean_ip_map<A: Ord + Copy>(ip_addr: A, ip_map: &IpAddrMap<A, Country>) -> String {
    // Safety: this function assumes a clean map.
    ip_map.search_unsafe(ip_addr).unwrap().name.to_string()
}

/// Lossily converts a char to a byte.
///
/// Where a char is multiple bytes, it returns only the first byte.
fn char_to_byte(char: char) -> u8 {
    // Convert the character into a `u32`, then take the first 8 bits
    // Safety: bit shift forces the `u32` to fit into `u8`
    (char as u32 >> (u32::BITS - u8::BITS)).try_into().unwrap()
}

/// For a given set of arguments, parse and return the IPv4 database into a clean `IpAddrMap`.
fn parse_ipv4(arguments: &Arguments) -> IpAddrMap<Ipv4Addr, Country> {
    // Safety: `arguments::get_config()` implements default values
    let ipv4_path = arguments.ipv4_path.clone().unwrap();
    let ipv4_len = arguments.ipv4_len.unwrap();
    let ipv4_comment = arguments.ipv4_comment.map(char_to_byte);

    let mut map = ip_geo::ipv4::parse_ipv4_file(ipv4_path, ipv4_len, ipv4_comment);
    map.cleanup();

    map
}

/// For a given set of arguments, parse and return the IPv6 database into an `IpAddrMap`.
fn parse_ipv6(arguments: &Arguments) -> IpAddrMap<Ipv6Addr, Country> {
    // Safety: `arguments::get_config()` implements default values
    let ipv6_path = arguments.ipv6_path.clone().unwrap();
    let ipv6_len = arguments.ipv6_len.unwrap();
    let ipv6_comment = arguments.ipv6_comment.map(char_to_byte);

    let mut map = ip_geo::ipv6::parse_ipv6_file(ipv6_path, ipv6_len, ipv6_comment);
    map.cleanup();

    map
}
