use serde::{Deserialize, Serialize};

// Die Feldnamen entsprechen der JSON-Antwort von Open-Meteo.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OrtSuche {
    #[serde(default)]
    pub results: Vec<Ort>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Ort {
    pub name: String,
    #[serde(default)]
    pub admin1: String,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub country_code: String,
    pub population: Option<u64>,
    pub latitude: f64,
    pub longitude: f64,
}

impl Ort {
    pub fn beschreibung(&self) -> String {
        [&self.name, &self.admin1, &self.country]
            .into_iter()
            .filter(|text| !text.is_empty())
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Wetter {
    pub timezone: String,
    pub current: Aktuell,
    pub daily: Tage,
    #[serde(default)]
    pub hourly: Stunden,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Aktuell {
    pub time: String,
    pub temperature_2m: Option<f64>,
    pub apparent_temperature: Option<f64>,
    pub relative_humidity_2m: Option<f64>,
    pub wind_speed_10m: Option<f64>,
    pub wind_gusts_10m: Option<f64>,
    pub wind_direction_10m: Option<f64>,
    pub cloud_cover: Option<f64>,
    pub pressure_msl: Option<f64>,
    pub visibility: Option<f64>,
    pub dew_point_2m: Option<f64>,
    pub weather_code: Option<u8>,
    pub is_day: Option<u8>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Tage {
    pub time: Vec<String>,
    pub temperature_2m_min: Vec<Option<f64>>,
    pub temperature_2m_max: Vec<Option<f64>>,
    pub precipitation_probability_max: Vec<Option<f64>>,
    pub precipitation_sum: Vec<Option<f64>>,
    pub uv_index_max: Vec<Option<f64>>,
    pub sunrise: Vec<Option<String>>,
    pub sunset: Vec<Option<String>>,
    pub daylight_duration: Vec<Option<f64>>,
    pub sunshine_duration: Vec<Option<f64>>,
    pub weather_code: Vec<Option<u8>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Stunden {
    pub time: Vec<String>,
    pub temperature_2m: Vec<Option<f64>>,
    pub precipitation_probability: Vec<Option<f64>>,
    pub precipitation: Vec<Option<f64>>,
    pub wind_speed_10m: Vec<Option<f64>>,
    pub weather_code: Vec<Option<u8>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Bericht {
    pub ort: Ort,
    pub wetter: Wetter,
}

impl Ort {
    pub fn kurz(&self) -> String {
        let kanton = match self.admin1.as_str() {
            "Kanton Zürich" | "Zürich" => "ZH",
            "Appenzell Ausserrhoden" | "Kanton Appenzell Ausserrhoden" => "AR",
            "Appenzell Innerrhoden" | "Kanton Appenzell Innerrhoden" => "AI",
            "Kanton St. Gallen" | "St. Gallen" => "SG",
            "Kanton Bern" | "Bern" => "BE",
            _ => self.country_code.as_str(),
        };
        if kanton.is_empty() {
            self.name.clone()
        } else {
            format!("{}, {kanton}", self.name)
        }
    }
}

impl Wetter {
    pub fn heute(&self) -> Option<usize> {
        self.daily
            .time
            .iter()
            .position(|tag| self.current.time.starts_with(tag))
    }
    pub fn tageswert(&self, werte: &[Option<f64>]) -> Option<f64> {
        self.heute().and_then(|i| werte.get(i).copied().flatten())
    }
    pub fn tageszeit<'a>(&self, werte: &'a [Option<String>]) -> Option<&'a str> {
        self.heute()
            .and_then(|i| werte.get(i).and_then(Option::as_deref))
    }
    pub fn stunden_indices(&self) -> Vec<usize> {
        let Some(stunde) = self.current.time.get(..13) else {
            return vec![];
        };
        self.hourly
            .time
            .iter()
            .enumerate()
            .filter(|(_, t)| t.as_str() >= stunde)
            .take(25)
            .map(|(i, _)| i)
            .collect()
    }
}
