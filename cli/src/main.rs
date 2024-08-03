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
use ip_geo::country::Country;

mod arguments;
use arguments::{Arguments, RunType};

fn main() {
    let arguments = arguments::get_config(Arguments::parse());

    match arguments::get_run_type(&arguments) {
        RunType::Ipv4 => print_country(find_ipv4(arguments)),
        RunType::Ipv6 => print_country(find_ipv6(arguments)),
        RunType::None => todo!("Trigger help message"),
    }
}

/// For a given `Country`, print ISO 3166-1 alpha-2 code and a country name (ex. `BE Belgium`).
fn print_country(country: Result<Country, ip_geo::Error>) {
    match country {
        Ok(country) => println!("{} {}", country.code, country.name),
        Err(error) => match error {
            ip_geo::Error::NoValueFound => println!("No country found!"),
            _ => eprintln!("{error}"),
        },
    }
}

/// For a given IPv4 address (contained in `arguments`), find the country it is associated with.
fn find_ipv4(arguments: Arguments) -> Result<Country, ip_geo::Error> {
    let comment = arguments.ipv4_comment.map(char_to_byte);

    let mut ipv4_map = ip_geo::ipv4::parse_ipv4_file(
        arguments
            .ipv4_path
            .expect("A valid path to an IPv4 GeoIP database"),
        arguments
            .ipv4_len
            .expect("The number of lines in the IPv4 GeoIP database"),
        comment,
    );

    let input_addr = arguments.ipv4_addr.expect("A valid IPv4 Address");

    ipv4_map.search(input_addr).cloned()
}

/// For a given IPv6 address (contained in `arguments`), find the country it is associated with.
fn find_ipv6(arguments: Arguments) -> Result<Country, ip_geo::Error> {
    let comment = arguments.ipv4_comment.map(char_to_byte);

    let mut ipv6_map = ip_geo::ipv6::parse_ipv6_file(
        arguments
            .ipv6_path
            .expect("A valid path to an IPv6 GeoIP database"),
        arguments
            .ipv6_len
            .expect("The number of lines in the IPv6 GeoIP database"),
        comment,
    );

    let input_addr = arguments.ipv6_addr.expect("A valid IPv6 Address");

    ipv6_map.search(input_addr).cloned()
}

/// Lossily converts a char to a byte.
///
/// Where a char is multiple bytes, it returns only the first byte.
fn char_to_byte(char: char) -> u8 {
    // How could this be improved?
    char.to_string().as_bytes().first().unwrap().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_ipv4() {
        use std::{io::Write, net::Ipv4Addr, path::Path};

        let start_a = Ipv4Addr::new(1, 1, 1, 1);
        let end_a = Ipv4Addr::new(3, 3, 3, 3);
        let value_a = "BE".into();
        let middle_a = Ipv4Addr::new(2, 2, 2, 2);

        let start_b = Ipv4Addr::new(4, 4, 4, 4);
        let end_b = Ipv4Addr::new(6, 6, 6, 6);
        let value_b = "CA".into();
        let middle_b = Ipv4Addr::new(5, 5, 5, 5);

        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        write!(
            temp_file,
            "{},{},{value_a}\n{},{},{value_b}\n",
            u32::from(start_a),
            u32::from(end_a),
            u32::from(start_b),
            u32::from(end_b),
        )
        .unwrap();
        let path: Box<Path> = temp_file.path().into();

        fn gen_args(addr: Ipv4Addr, path: Box<Path>) -> arguments::Arguments {
            Arguments {
                config_path: None,
                ipv4_addr: Some(addr),
                ipv4_path: Some(path),
                ipv4_len: Some(2),
                ipv4_comment: None,
                ipv6_addr: None,
                ipv6_path: None,
                ipv6_len: None,
                ipv6_comment: None,
            }
        }

        fn get_code(addr: Ipv4Addr, path: Box<Path>) -> Box<str> {
            find_ipv4(gen_args(addr, path)).unwrap().code
        }

        assert_eq!(get_code(middle_a, path.clone()), value_a);
        assert_eq!(get_code(middle_b, path), value_b);
    }

    #[test]
    fn test_find_ipv6() {
        use std::{io::Write, net::Ipv6Addr, path::Path, str::FromStr};

        let start_a = "1::";
        let end_a = "3::";
        let value_a = "BE".into();
        let middle_a = Ipv6Addr::from_str("2::").unwrap();

        let start_b = "4::";
        let end_b = "6::";
        let value_b = "CA".into();
        let middle_b = Ipv6Addr::from_str("5::").unwrap();

        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        write!(
            temp_file,
            "{},{},{value_a}\n{},{},{value_b}\n",
            start_a, end_a, start_b, end_b,
        )
        .unwrap();
        let path: Box<Path> = temp_file.path().into();

        fn gen_args(addr: Ipv6Addr, path: Box<Path>) -> arguments::Arguments {
            Arguments {
                config_path: None,
                ipv4_addr: None,
                ipv4_path: None,
                ipv4_len: None,
                ipv4_comment: None,
                ipv6_addr: Some(addr),
                ipv6_path: Some(path),
                ipv6_len: Some(2),
                ipv6_comment: None,
            }
        }

        fn get_code(addr: Ipv6Addr, path: Box<Path>) -> Box<str> {
            find_ipv6(gen_args(addr, path)).unwrap().code
        }

        assert_eq!(get_code(middle_a, path.clone()), value_a);
        assert_eq!(get_code(middle_b, path.clone()), value_b);
    }
}
