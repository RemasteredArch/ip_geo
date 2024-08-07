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

use std::sync::Arc;

use serde::Serialize;

use crate::country_list::Country;

impl PartialEq for Country {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code // && self.name == other.name
    }
}

#[derive(Serialize)]
struct SerializableCountry {
    code: Box<str>,
    name: Box<str>,
    coordinates: (f64, f64),
}

impl SerializableCountry {
    fn new(code: Box<str>, name: Box<str>, coordinates: (f64, f64)) -> Self {
        Self {
            code,
            name,
            coordinates,
        }
    }
}

impl From<Country> for SerializableCountry {
    fn from(value: Country) -> Self {
        let to_box = |s: Arc<str>| s.to_string().into_boxed_str();

        SerializableCountry::new(to_box(value.code), to_box(value.name), value.coordinates)
    }
}

impl From<&Country> for SerializableCountry {
    fn from(value: &Country) -> Self {
        let to_box = |s: &Arc<str>| s.clone().to_string().into_boxed_str();

        SerializableCountry::new(to_box(&value.code), to_box(&value.name), value.coordinates)
    }
}

impl Serialize for Country {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializableCountry::from(self).serialize(serializer)
    }
}
