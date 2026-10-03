use crate::data::*;
use chrono::{Duration, NaiveDateTime};

pub fn beispiel() -> Bericht {
    let start = NaiveDateTime::parse_from_str("2026-10-03T16:00", "%Y-%m-%dT%H:%M").unwrap();
    Bericht {
        ort: Ort {
            name: "Schönengrund".into(),
            admin1: "Appenzell Ausserrhoden".into(),
            country: "Schweiz".into(),
            country_code: "CH".into(),
            ..Ort::default()
        },
        wetter: Wetter {
            timezone: "Europe/Zurich".into(),
            current: Aktuell {
                time: "2026-10-03T16:45".into(),
                temperature_2m: Some(19.3),
                apparent_temperature: Some(19.0),
                wind_speed_10m: Some(5.6),
                wind_gusts_10m: Some(13.3),
                wind_direction_10m: Some(315.0),
                relative_humidity_2m: Some(68.0),
                dew_point_2m: Some(13.2),
                cloud_cover: Some(45.0),
                visibility: Some(24000.0),
                pressure_msl: Some(1018.0),
                weather_code: Some(2),
                is_day: Some(1),
            },
            hourly: Stunden {
                time: (0..25)
                    .map(|i| {
                        (start + Duration::hours(i))
                            .format("%Y-%m-%dT%H:%M")
                            .to_string()
                    })
                    .collect(),
                temperature_2m: (0..25).map(|i| Some(19.5 - i as f64 * 0.3)).collect(),
                precipitation_probability: vec![Some(20.0); 25],
                precipitation: vec![Some(0.2); 25],
                wind_speed_10m: vec![Some(6.0); 25],
                weather_code: vec![Some(3); 25],
            },
            daily: Tage {
                time: vec![
                    "2026-10-03".into(),
                    "2026-10-04".into(),
                    "2026-10-05".into(),
                ],
                temperature_2m_min: vec![Some(11.1); 3],
                temperature_2m_max: vec![Some(19.5); 3],
                precipitation_probability_max: vec![Some(20.0); 3],
                precipitation_sum: vec![Some(1.5); 3],
                sunset: vec![Some("2026-10-03T18:58".into())],
                sunrise: vec![Some("2026-10-03T07:24".into())],
                daylight_duration: vec![Some(41640.0)],
                sunshine_duration: vec![Some(21600.0)],
                uv_index_max: vec![Some(3.2)],
                weather_code: vec![Some(3); 3],
            },
        },
    }
}
