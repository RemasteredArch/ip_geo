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

use std::collections::HashMap;

/// Represents a country.
#[derive(Clone, Debug)]
pub struct Country {
    /// The two letter country code.
    ///
    /// Ex. "BE" for Belgium.
    pub code: Box<str>,

    /// The full name of the country.
    ///
    /// Ex. "Belgium".
    pub name: Box<str>,
}

impl Country {
    /// Creates a country from a country code and a map of <country code, country name>.
    pub fn from_code(code: &str, country_map: &HashMap<String, String>) -> Option<Self> {
        let (code, name) = country_map.get_key_value(code)?;

        Some(Country {
            code: code.clone().into_boxed_str(),
            name: name.clone().into_boxed_str(),
        })
    }
}

impl PartialEq for Country {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code // && self.name == other.name
    }
}
