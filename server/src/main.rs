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
    str::FromStr,
};

use warp::Filter;

mod arguments;
use arguments::Arguments;

#[tokio::main]
pub async fn main() {
    let arguments = arguments::get_config(Arguments::parse());

    // Safety: `crate::arguments` implements default values
    let config_path = arguments.config_path.unwrap();
    let port = arguments.port.unwrap();

    let root = warp::path("v1");
    let get = root.and(warp::path("get"));

    let ipv4_map = parse_ipv4(arguments);
    let ipv6_map = parse_ipv6(arguments);

    let ipv4 = get
        .and(warp::path!("ipv4" / Ipv4Addr))
        .map(|ipv4_addr| todo!());

    println!("Serving on http://127.0.0.1:{port}/");

    warp::serve(ipv4).run(([127, 0, 0, 1], port)).await;
}

/// Lossily converts a char to a byte.
///
/// Where a char is multiple bytes, it returns only the first byte.
fn char_to_byte(char: char) -> u8 {
    // How could this be improved?
    char.to_string().as_bytes().first().unwrap().to_owned()
}

/// For a given set of arguments, parse and return the IPv4 database into an `IpAddrMap`.
fn parse_ipv4(arguments: Arguments) -> IpAddrMap<Ipv4Addr, Country> {
    // Safety: `crate::arguments` implements default values
    let ipv4_path = arguments.ipv4_path.unwrap();
    let ipv4_len = arguments.ipv4_len.unwrap();
    let ipv4_comment = arguments.ipv4_comment.map(char_to_byte);

    ip_geo::ipv4::parse_ipv4_file(ipv4_path, ipv4_len, ipv4_comment)
}

/// For a given set of arguments, parse and return the IPv6 database into an `IpAddrMap`.
fn parse_ipv6(arguments: Arguments) -> IpAddrMap<Ipv6Addr, Country> {
    // Safety: `crate::arguments` implements default values
    let ipv6_path = arguments.ipv6_path.unwrap();
    let ipv6_len = arguments.ipv6_len.unwrap();
    let ipv6_comment = arguments.ipv6_comment.map(char_to_byte);

    ip_geo::ipv6::parse_ipv6_file(ipv6_path, ipv6_len, ipv6_comment)
}
