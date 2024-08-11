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

use std::net::{Ipv4Addr, Ipv6Addr};

use ip_geo::{country_list::Country, IpAddrMap};
use serde::Serialize;
use warp::{
    http::StatusCode,
    reply::{json, with_status, Json, WithStatus},
    Filter, Rejection, Reply,
};

use crate::parse::Maps;

pub static API_VERSION: &str = "v0";

/// For a give Warp routes map, and a list of target `SocketAddr`s, print the targets and serve the
/// routes on them.
macro_rules! serve {
    ( $routes:expr, $( $target:expr ),+ ) => {
        ::tokio::join!(
            $({
                println!("Serving on http://{}/{}/", $target, $crate::api::API_VERSION);
                ::warp::serve($routes.clone()).run($target)
            }),+
        );
    };
}

pub fn get_routes(maps: Maps) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let search_ipv4 = move |ipv4_addr: Ipv4Addr| search_clean_ip_map(ipv4_addr, &maps.v4);
    let search_ipv6 = move |ipv6_addr: Ipv6Addr| search_clean_ip_map(ipv6_addr, &maps.v6);

    let ipv4 = warp::path!("ipv4" / Ipv4Addr).map(search_ipv4);
    let ipv6 = warp::path!("ipv6" / Ipv6Addr).map(search_ipv6);

    warp::get().and(warp::path(API_VERSION)).and(ipv4.or(ipv6))
}

/// Search an IPv4 address map for an IP address.
///
/// Assumes that the `IpAddrMap` is clean, otherwise it return an internal server error (code 500).
fn search_clean_ip_map<A: Ord + Copy>(ip_addr: A, ip_map: &IpAddrMap<A, Country>) -> impl Reply {
    fn success(country: &Country) -> WithStatus<Json> {
        json_with_status(country, StatusCode::OK)
    }

    fn error(error: ip_geo::Error) -> WithStatus<Json> {
        match error {
            ip_geo::Error::NoValueFound => json_str_error(
                "no country associated with IP address",
                StatusCode::NOT_FOUND,
            ),
            _ => {
                eprintln!("Error 500: request resulted in error: '{error}'");
                json_str_error(&error.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    match ip_map.try_search(ip_addr) {
        Ok(country) => success(country),
        Err(err) => error(err),
    }
}

/// Returns a JSON reply with a given status.
///
/// Returns JSON in the format of:
///
/// ```json
/// {"error":"example error text"}
/// ```
fn json_str_error(error: &str, code: StatusCode) -> WithStatus<Json> {
    #[derive(Serialize)]
    struct SerializableError<'s> {
        error: &'s str,
    }

    json_with_status(&SerializableError { error }, code)
}

/// Returns a JSON reply with the given contents and status code.
fn json_with_status(contents: &impl Serialize, code: StatusCode) -> WithStatus<Json> {
    with_status(json(contents), code)
}
