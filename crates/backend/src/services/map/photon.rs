use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Place {
    pub name: String,
    pub detail: String,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Deserialize)]
struct Collection {
    #[serde(default)]
    features: Vec<Feature>,
}

#[derive(Deserialize)]
struct Feature {
    geometry: Geometry,
    #[serde(default)]
    properties: Properties,
}

#[derive(Deserialize)]
struct Geometry {
    coordinates: (f64, f64),
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Properties {
    name: Option<String>,
    street: Option<String>,
    housenumber: Option<String>,
    city: Option<String>,
    district: Option<String>,
    county: Option<String>,
    state: Option<String>,
    country: Option<String>,
}

fn present(value: Option<&String>) -> Option<&str> {
    value
        .map(|text| text.trim())
        .filter(|text| !text.is_empty())
}

impl Properties {
    fn name(&self) -> Option<String> {
        if let Some(name) = present(self.name.as_ref()) {
            return Some(name.to_string());
        }
        let street = present(self.street.as_ref())?;
        Some(match present(self.housenumber.as_ref()) {
            Some(number) => format!("{street} {number}"),
            None => street.to_string(),
        })
    }

    fn detail(&self, name: &str) -> String {
        let mut parts: Vec<&str> = Vec::new();
        for part in [
            &self.district,
            &self.city,
            &self.county,
            &self.state,
            &self.country,
        ] {
            let Some(text) = present(part.as_ref()) else {
                continue;
            };
            if text != name && !parts.contains(&text) {
                parts.push(text);
            }
        }
        parts.join(", ")
    }
}

fn valid(lat: f64, lon: f64) -> bool {
    lat.is_finite() && lon.is_finite() && lat.abs() <= 90.0 && lon.abs() <= 180.0
}

pub(super) fn parse(body: &[u8]) -> Result<Vec<Place>, serde_json::Error> {
    let collection: Collection = serde_json::from_slice(body)?;
    Ok(collection
        .features
        .into_iter()
        .filter_map(|feature| {
            let (lon, lat) = feature.geometry.coordinates;
            if !valid(lat, lon) {
                return None;
            }
            let name = feature.properties.name()?;
            let detail = feature.properties.detail(&name);
            Some(Place {
                name,
                detail,
                lat,
                lon,
            })
        })
        .collect())
}
