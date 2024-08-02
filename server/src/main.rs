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
use std::{net::Ipv4Addr, str::FromStr};

use warp::Filter;

mod arguments;
use arguments::Arguments;

#[tokio::main]
pub async fn main() {
    let arguments = arguments::get_config(Arguments::parse());

    // Safety: `crate::arguments` implements default values
    let config_path = arguments.config_path.unwrap();
    let ipv4_path = arguments.ipv4_path.unwrap();
    let ipv4_len = arguments.ipv4_len.unwrap();
    let ipv4_comment = arguments.ipv4_comment.unwrap();
    let ipv6_path = arguments.ipv6_path.unwrap();
    let ipv6_len = arguments.ipv6_len.unwrap();
    let ipv6_comment = arguments.ipv6_comment.unwrap();
    let port = arguments.port.unwrap();

    let root = warp::path("v1");
    let get = root.and(warp::path("get"));

    let ipv4 = get
        .and(warp::path!("ipv4" / Ipv4Addr))
        .map(|ipv4_addr| todo!());

    println!("Serving on http://127.0.0.1:{port}/");

    warp::serve(ipv4).run(([127, 0, 0, 1], port)).await;
}
