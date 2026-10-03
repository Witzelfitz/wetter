pub fn wert(zahl: Option<f64>, einheit: &str) -> String {
    match zahl.filter(|n| n.is_finite()) {
        Some(n) => format!("{n:.1}{einheit}"),
        None => "–".to_owned(),
    }
}

pub fn wettertext(code: Option<u8>) -> &'static str {
    match code {
        Some(0) => "Klar",
        Some(1) => "Überwiegend klar",
        Some(2) => "Teilweise bewölkt",
        Some(3) => "Bedeckt",
        Some(45 | 48) => "Nebel",
        Some(51 | 53 | 55) => "Nieselregen",
        Some(56 | 57 | 66 | 67) => "Gefrierender Regen",
        Some(61 | 63 | 65) => "Regen",
        Some(71 | 73 | 75 | 77) => "Schnee",
        Some(80..=82) => "Regenschauer",
        Some(85 | 86) => "Schneeschauer",
        Some(95..=97 | 99) => "Gewitter",
        _ => "Keine Wetterangabe",
    }
}

pub fn ganz(zahl: Option<f64>, einheit: &str) -> String {
    zahl.filter(|v| v.is_finite())
        .map_or_else(|| "–".to_owned(), |v| format!("{v:.0}{einheit}"))
}

pub fn dauer(sekunden: Option<f64>) -> String {
    match sekunden.filter(|s| s.is_finite() && *s >= 0.0) {
        Some(s) => {
            let minuten = (s / 60.0).round() as u64;
            format!("{}h {:02}m", minuten / 60, minuten % 60)
        }
        None => "–".to_owned(),
    }
}

pub fn uhrzeit(iso: Option<&str>) -> &str {
    iso.and_then(|t| t.get(11..16))
        .filter(|t| t.as_bytes().get(2) == Some(&b':'))
        .unwrap_or("–")
}

pub fn datum_kurz(iso: &str) -> String {
    match (iso.get(8..10), iso.get(5..7)) {
        (Some(tag), Some(monat)) => format!("{tag}.{monat}."),
        _ => iso.to_owned(),
    }
}

pub fn windrichtung(grad: Option<f64>) -> &'static str {
    const RICHTUNGEN: [&str; 8] = ["N", "NO", "O", "SO", "S", "SW", "W", "NW"];
    grad.filter(|g| g.is_finite()).map_or("–", |g| {
        RICHTUNGEN[((g.rem_euclid(360.0) / 45.0).round() as usize) % 8]
    })
}

pub fn wetterbild(code: Option<u8>, nacht: bool) -> ([&'static str; 5], u8) {
    match code {
        Some(0 | 1) if nacht => (
            [
                "     _..._    .",
                "   .'    _`.     *",
                "  /    .'  `",
                "  \\    `-._   .",
                "   `-....-'",
            ],
            94,
        ),
        Some(0 | 1) => (
            [
                "      \\   /",
                "    .- .-. -.",
                "  --  (   )  --",
                "    '- `-' -'",
                "      /   \\",
            ],
            93,
        ),
        Some(2) => (
            [
                "   \\ | /",
                " -- ( ) .--.",
                "   / | (    ).",
                "      (____(__)",
                "",
            ],
            93,
        ),
        Some(3) => (
            [
                "       .--.",
                "    .-(    ).",
                "   (        ) )",
                "    `------'-'",
                "",
            ],
            37,
        ),
        Some(45 | 48) => (
            [
                "  _ - _ - _ - _",
                "   _ - _ - _ -",
                "  _ - _ - _ - _",
                "   _ - _ - _ -",
                "  _ - _ - _ - _",
            ],
            37,
        ),
        Some(51..=67 | 80..=82) => (
            [
                "      .--.",
                "   .-(    ).",
                "  (___(__)__)",
                "    / / / /",
                "   / / / /",
            ],
            94,
        ),
        Some(71..=77 | 85 | 86) => (
            [
                "      .--.",
                "   .-(    ).",
                "  (___(__)__)",
                "    *  +  *",
                "   +  *  +",
            ],
            96,
        ),
        Some(95..=97 | 99) => (
            [
                "      .--.",
                "   .-(    ).",
                "  (___(__)__)",
                "    / /_/ /",
                "   /  /  /",
            ],
            93,
        ),
        _ => (["", "      .---.", "      | ? |", "      '---'", ""], 90),
    }
}
