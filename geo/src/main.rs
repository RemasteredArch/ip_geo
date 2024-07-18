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

#![allow(dead_code)]

use mediawiki::{reqwest::Url, ApiSync, MediaWikiError};
use serde_json::Value;

fn main() {
    get_country_list();
}

struct Country {
    id: String,      // Ex. Q31
    id_url: Url,     // Ex. http://www.wikidata.org/entity/Q31
    country: String, // Ex. Belgium
    code: String,    // Ex. BE
}

impl Country {
    fn new_from_query() -> Self {
        todo!();
    }
}

fn get_country_list() {
    let query = r#"
SELECT
    ?country      # Ex. http://www.wikidata.org/entity/Q31
    ?countryLabel # Ex. Belgium
    ?code         # Ex. BE
WHERE
{
    ?country wdt:P31 wd:Q6256;  # For every instance of (p:31) country (wq:Q6256)
        wdt:P297 ?code.         # Get its ISO 3166-1 alpha-2 code (P297)

    SERVICE wikibase:label { bd:serviceParam wikibase:language "en". } # Or "[AUTO_LANGUAGE],en"
}
# LIMIT 300
"#;

    let wikidata_query = wikidata_query(query).expect("The result of a Wikidata Query");

    dbg!(wikidata_query);

    //dbg!(get_value(&result[0], "code"));
}

fn get_value<'st>(result: &'st Value, label: &'st str) -> Option<&'st Value> {
    result.as_object()?.get(label)?.get("value")
}

fn wikidata_query(query: &str) -> Option<Vec<Value>> {
    Some(
        ApiSync::new("https://www.wikidata.org/w/api.php")
            .ok()?
            .sparql_query(query)
            .ok()?
            .as_object()?
            .to_owned()
            .get("results")?
            .get("bindings")?
            .as_array()?
            .to_owned(),
    )
}
