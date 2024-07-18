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

use std::str::FromStr;

use mediawiki::{reqwest::Url, ApiSync};
use serde_json::Value;

fn main() {
    let mut additional_countries = vec![
        Country::new_without_id("AP", "African Regional Intellectual Property Organization"),
        Country::new_without_id("CS", "Serbia and Montenegro"),
    ];
    let countries = get_country_list(&mut additional_countries);

    dbg!(countries);
}

#[derive(Debug)]
#[allow(dead_code)]
struct Country {
    id: Option<String>,  // Ex. Q31
    id_url: Option<Url>, // Ex. http://www.wikidata.org/entity/Q31
    country: String,     // Ex. Belgium
    code: String,        // Ex. BE
}

impl Country {
    fn new_without_id(code: &str, name: &str) -> Self {
        Self {
            id: None,
            id_url: None,
            country: name.to_string(),
            code: code.to_string(),
        }
    }

    fn new_from_query(country_result: Value) -> Option<Self> {
        let url_str = get_value(&country_result, "country")?.as_str()?;

        let id_url = Some(Url::from_str(url_str).ok()?);
        let id = Some(id_url.clone()?.path_segments()?.last()?.to_owned());

        let country = get_value(&country_result, "countryLabel")?
            .as_str()?
            .to_string();

        let code = get_value(&country_result, "code")?.as_str()?.to_string();

        Some(Self {
            id,
            id_url,
            country,
            code,
        })
    }
}

fn get_country_list(additional_countries: &mut Vec<Country>) -> Vec<Country> {
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

    let result = wikidata_query(query).expect("The result of a Wikidata Query");

    let mut countries = Vec::with_capacity(result.len() + additional_countries.len());

    for country in result {
        countries.push(Country::new_from_query(country).unwrap());
    }

    countries.append(additional_countries);
    countries.dedup_by_key(|c| c.code.clone());

    countries
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
