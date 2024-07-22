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

#![allow(dead_code)]

use clap::Parser;
use ip_geo::{parse_ipv4_file, parse_ipv6_file};

mod arguments;
use arguments::{Arguments, RunType};

fn main() {
    let arguments = arguments::get_config(Arguments::parse());

    match arguments::get_run_type(&arguments) {
        RunType::Server => launch_server(arguments),
        RunType::Ipv4 => find_ipv4(arguments),
        RunType::Ipv6 => find_ipv6(arguments),
        RunType::None => todo!("Trigger help message"),
    }
}

/// For a given IPv4 address (contained in `arguments`), find the country it is associated with.
///
/// Prints an ISO 3166-1 alpha-2 code (ex. BE for Belgium").
fn find_ipv4(arguments: Arguments) {
    let mut ipv4_map = parse_ipv4_file(
        arguments
            .ipv4_path
            .expect("A valid path to an IPv4 GeoIP database"),
        arguments
            .ipv4_len
            .expect("The number of lines in the IPv4 GeoIP database"),
    );

    let input_addr = arguments.ipv4_addr.expect("A valid IPv4 Address");
    dbg!(input_addr);

    if let Some(result) = ipv4_map.search(input_addr) {
        dbg!(result);
        println!("{}", result.long_name);
    } else {
        println!("No match!");
    }
}

/// For a given IPv6 address (contained in `arguments`), find the country it is associated with.
///
/// Prints an ISO 3166-1 alpha-2 code (ex. BE for Belgium").
fn find_ipv6(arguments: Arguments) {
    let mut ipv6_map = parse_ipv6_file(
        arguments
            .ipv6_path
            .expect("A valid path to an IPv6 GeoIP database"),
        arguments
            .ipv6_len
            .expect("The number of lines in the IPv6 GeoIP database"),
    );

    let input_addr = arguments.ipv6_addr.expect("A valid IPv6 Address");
    dbg!(input_addr);

    if let Some(result) = ipv6_map.search(input_addr) {
        dbg!(result);
        println!("{}", result.long_name);
    } else {
        println!("No match!");
    }
}

/// Launch an HTTP server that can respond to requests to resolve IP addresses to countries
fn launch_server(arguments: Arguments) {
    todo!("Implement the server functionality");
}
