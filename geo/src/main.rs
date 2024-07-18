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

use std::{str::FromStr, string::ParseError};

use mediawiki::{reqwest::Url, ApiSync, MediaWikiError};
use serde_json::Value;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error(transparent)]
    Url(#[from] ParseError),
    #[error(transparent)]
    Wiki(#[from] MediaWikiError),
    #[error("can't map value to object")]
    InvalidObject,
    #[error("can't map value to array")]
    InvalidArray,
    #[error("missing results in response")]
    MissingResults,
    #[error("missing binding in value")]
    MissingBindings,
}

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
    id: Option<Box<str>>, // Ex. Q31
    id_url: Option<Url>,  // Ex. http://www.wikidata.org/entity/Q31
    country: Box<str>,    // Ex. Belgium
    code: Box<str>,       // Ex. BE
}

impl Country {
    fn new_without_id(code: impl AsRef<str>, name: impl AsRef<str>) -> Self {
        Self {
            id: None,
            id_url: None,
            country: name.as_ref().into(),
            code: code.as_ref().into(),
        }
    }

    fn new_from_query(country_result: Value) -> Result<Self, Error> {
        let url_str = get_value(&country_result, "country")?.as_str()?; // throw invalid string

        let id_url = Some(Url::from_str(url_str)?);
        let id = Some(id_url.clone()?.path_segments()?.last()?.into());

        let country = get_value(&country_result, "countryLabel")?.as_str()?.into();

        let code = get_value(&country_result, "code")?.as_str()?.into();

        Some(Self {
            id,
            id_url,
            country,
            code,
        })
    }
}

fn get_country_list(additional_countries: &mut Vec<Country>) -> Box<[Country]> {
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

    countries.into_boxed_slice()
}

fn get_value<'st>(result: &'st Value, label: &str) -> Result<&'st Value, Error> {
    result
        .as_object()
        .ok_or(Error::InvalidObject)?
        .get(label)
        .ok_or(Error::MissingBindings)?
        .get("value")
        .ok_or(Error::MissingBindings)
}

fn wikidata_query(query: &str) -> Result<Vec<Value>, Error> {
    Ok(ApiSync::new("https://www.wikidata.org/w/api.php")?
        .sparql_query(query)?
        .as_object()
        .ok_or(Error::InvalidObject)?
        .to_owned()
        .get("results")
        .ok_or(Error::MissingResults)?
        .get("bindings")
        .ok_or(Error::MissingBindings)?
        .as_array()
        .ok_or(Error::InvalidArray)?
        .to_owned())
}
