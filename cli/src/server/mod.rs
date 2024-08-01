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

use std::{net::Ipv4Addr, str::FromStr};

use warp::Filter;

use crate::{arguments::Arguments, find_ipv4};

pub async fn launch_server(arguments: Arguments) {
    // Safety: `crate::arguments` implements default values
    let port = arguments.port.unwrap();

    let root = warp::path("v1");
    let get = root.and(warp::path("get"));

    let ipv4 = get
        .and(warp::path("ipv4"))
        .and(warp::path::param())
        .map(move |ipv4_addr: String| get_ipv4(&ipv4_addr, arguments.clone()));

    println!("Serving on http://127.0.0.1:{port}/");

    warp::serve(ipv4).run(([127, 0, 0, 1], port)).await;
}

fn get_ipv4(ipv4_addr: &str, mut arguments: Arguments) -> String {
    arguments.ipv4_addr = Ipv4Addr::from_str(ipv4_addr).ok();

    match find_ipv4(arguments) {
        Some(country) => country.name.to_string(),
        None => "<No Country Found!>".to_string(),
    }
}
