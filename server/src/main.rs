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

#[macro_use]
mod api;

mod arguments;
use arguments::Arguments;

mod error;

mod parse;

#[tokio::main]
pub async fn main() {
    // Parse options
    let arguments = arguments::get_config(Arguments::parse());

    // Safety: `arguments::get_config()` implements default values
    let ipv4_target = arguments.ipv4_pair.unwrap();
    let ipv6_target = arguments.ipv6_pair.unwrap();

    // Parse databases
    let maps = parse::parse_ip_maps(&arguments);

    // Construct routes
    let routes = api::get_routes(maps);

    // Serve routes
    serve!(routes, ipv4_target, ipv6_target);
}
