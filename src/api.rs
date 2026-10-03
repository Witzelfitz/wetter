use crate::data::*;
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use std::time::Duration;

pub fn client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())
}

pub fn suchen(client: &Client, name: &str) -> Result<Vec<Ort>, String> {
    if name.trim().chars().count() < 2 {
        return Err("Bitte mindestens zwei Zeichen eingeben.".into());
    }
    let name = suchname(name);
    let mut suche: OrtSuche = abrufen(
        client,
        "https://geocoding-api.open-meteo.com/v1/search",
        &[("name", name.as_str()), ("count", "20"), ("language", "de")],
    )?;
    suche
        .results
        .sort_by_key(|ort| std::cmp::Reverse(ort.population.unwrap_or(0)));
    if suche.results.is_empty() {
        return Err("Keinen Ort gefunden. Bitte Schreibweise prüfen.".into());
    }
    Ok(suche.results)
}

// GeoNames erwartet bei Abkürzungen wie St.Gallen ein Leerzeichen nach dem Punkt.
fn suchname(name: &str) -> String {
    let mut result = String::new();
    let mut chars = name.trim().chars().peekable();
    while let Some(c) = chars.next() {
        result.push(c);
        if c == '.' && chars.peek().is_some_and(|c| c.is_alphabetic()) {
            result.push(' ');
        }
    }
    result
}

pub fn wetter(client: &Client, ort: &Ort) -> Result<Wetter, String> {
    abrufen(
        client,
        "https://api.open-meteo.com/v1/forecast",
        &[
            ("latitude", &ort.latitude.to_string()),
            ("longitude", &ort.longitude.to_string()),
            ("timezone", "auto"),
            ("forecast_days", "3"),
            ("temperature_unit", "celsius"),
            ("wind_speed_unit", "kmh"),
            ("precipitation_unit", "mm"),
            (
                "current",
                "temperature_2m,apparent_temperature,relative_humidity_2m,wind_speed_10m,wind_gusts_10m,wind_direction_10m,cloud_cover,pressure_msl,visibility,dew_point_2m,weather_code,is_day",
            ),
            (
                "hourly",
                "temperature_2m,precipitation_probability,precipitation,wind_speed_10m,weather_code",
            ),
            (
                "daily",
                "temperature_2m_min,temperature_2m_max,precipitation_probability_max,precipitation_sum,uv_index_max,sunrise,sunset,daylight_duration,sunshine_duration,weather_code",
            ),
        ],
    )
}

fn abrufen<T: DeserializeOwned>(
    client: &Client,
    url: &str,
    parameter: &[(&str, &str)],
) -> Result<T, String> {
    let antwort = client
        .get(url)
        .query(parameter)
        .send()
        .map_err(|e| format!("API nicht erreichbar. Prüfe deine Internetverbindung. ({e})"))?
        .error_for_status()
        .map_err(|e| format!("Die API meldet einen Fehler: {e}"))?;
    antwort
        .json()
        .map_err(|e| format!("Die Wetterdaten konnten nicht gelesen werden: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ortsabkuerzungen_werden_ohne_verlust_normalisiert() {
        assert_eq!(suchname(" St.Gallen "), "St. Gallen");
        assert_eq!(suchname("St. Gallen"), "St. Gallen");
        assert_eq!(suchname("St.Moritz"), "St. Moritz");
        assert_eq!(suchname("Zürich"), "Zürich");
        assert_eq!(suchname("9000"), "9000");
    }
}
