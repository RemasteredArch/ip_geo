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
use ip_geo::{country_list::Country, IpAddrMap};
use serde_derive::Serialize;
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    sync::Arc,
};
use warp::{
    http::StatusCode,
    reply::{with_status, WithStatus},
    Reply,
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
/// Assumes that the `IpAddrMap` is clean, otherwise it return an internal server error (code 500).
fn search_clean_ip_map<A: Ord + Copy>(ip_addr: A, ip_map: &IpAddrMap<A, Country>) -> impl Reply {
    fn success(country: &Country) -> WithStatus<String> {
        with_status(country.name.to_string(), StatusCode::OK)
    }

    fn error(error: ip_geo::Error) -> WithStatus<String> {
        match error {
            ip_geo::Error::NoValueFound => with_status(
                "no country associated with IP address".to_string(),
                StatusCode::NOT_FOUND,
            ),
            _ => {
                eprintln!("Error 500: request resulted in error: '{error}'");
                with_status(error.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    match ip_map.try_search(ip_addr) {
        Ok(country) => success(country),
        Err(err) => error(err),
    }
}

/// For a given set of arguments, parse and return the IPv4 database into a clean `IpAddrMap`.
fn parse_ipv4(arguments: &Arguments) -> IpAddrMap<Ipv4Addr, Country> {
    // Safety: `arguments::get_config()` implements default values
    let path = arguments.ipv4_path.clone().unwrap();
    let file_length = arguments.ipv4_len.unwrap();
    let comment = arguments.ipv4_comment;

    let mut map = ip_geo::ipv4::parse_ipv4_file(path, file_length, comment);
    map.cleanup();

    map
}

/// For a given set of arguments, parse and return the IPv6 database into an `IpAddrMap`.
fn parse_ipv6(arguments: &Arguments) -> IpAddrMap<Ipv6Addr, Country> {
    // Safety: `arguments::get_config()` implements default values
    let path = arguments.ipv6_path.clone().unwrap();
    let file_length = arguments.ipv6_len.unwrap();
    let comment = arguments.ipv6_comment;

    let mut map = ip_geo::ipv6::parse_ipv6_file(path, file_length, comment);
    map.cleanup();

    map
}

#[derive(Serialize)]
struct CountryApi {
    code: Box<str>,
    name: Box<str>,
}

impl CountryApi {
    fn new(code: Box<str>, name: Box<str>) -> Self {
        Self { code, name }
    }
}

impl From<Country> for CountryApi {
    fn from(value: Country) -> Self {
        let to_box = |s: Arc<str>| s.to_string().into_boxed_str();

        CountryApi::new(to_box(value.code), to_box(value.name))
    }
}
