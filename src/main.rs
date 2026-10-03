mod api;
mod app;
mod data;
mod format;
#[cfg(test)]
mod test_support;
mod theme;
mod ui;

use data::*;
use ratatui::{Terminal, backend::TestBackend, style::Color};
use std::io::{self, IsTerminal, Write};
use unicode_width::UnicodeWidthStr;

type Ergebnis<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Modus {
    Einmal,
    Tui,
    Plain,
    Json,
}

fn main() {
    if let Err(e) = starten() {
        eprintln!("Fehler: {e}");
        std::process::exit(1);
    }
}

fn starten() -> Ergebnis<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let flags: Vec<_> = args.iter().take_while(|s| s.as_str() != "--").collect();
    if flags
        .iter()
        .any(|s| s.as_str() == "--version" || s.as_str() == "-V")
    {
        println!("wetter {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if flags
        .iter()
        .any(|s| s.as_str() == "--help" || s.as_str() == "-h")
    {
        println!(
            "WETTER – dein Wetter im Terminal\n\nwetter Zürich            Einmaliger Wetterbericht\nwetter --tui Zürich      Interaktive Vollbild-App\nwetter --plain Zürich    ASCII-Text, ohne Farben\nwetter --json Zürich     JSON für Skripte\nwetter --pick 2 Zürich   Zweiten Ort aus der Suchliste verwenden\nwetter --version         Versionsnummer anzeigen\nwetter --help            Diese Hilfe anzeigen\n\nOhne Ortsangabe: Eingabeaufforderung (JSON benötigt einen Ort).\nJSON und Pipes wählen standardmässig den grössten Treffer.\nIn der TUI: Tab / 1 2 3 = Ansicht, / = Ort, r = aktualisieren, q = Ende.\nNO_COLOR deaktiviert Farben. Internetverbindung erforderlich."
        );
        return Ok(());
    }
    let (modus, mut name, pick) = argumente(&args)?;
    if modus == Modus::Tui {
        if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
            return Err(
                "--tui benötigt ein interaktives Terminal. Nutze --plain oder --json.".into(),
            );
        }
        if pick.is_some() {
            return Err(
                "--pick ist für die einmalige Ausgabe. In --tui wählst du den Ort aus der Liste."
                    .into(),
            );
        }
        app::tui(name)?;
        return Ok(());
    }
    if name.is_none() {
        if modus == Modus::Json || !io::stdin().is_terminal() {
            return Err("Bitte einen Ort angeben, z. B. wetter --json Zürich.".into());
        }
        eprint!("Ort: ");
        io::stderr().flush()?;
        let mut text = String::new();
        io::stdin().read_line(&mut text)?;
        name = Some(text.trim().to_owned());
    }
    let client = api::client()?;
    let orte = api::suchen(&client, name.as_deref().unwrap_or(""))?;
    let index = if let Some(n) = pick {
        n - 1
    } else if modus != Modus::Json
        && io::stdin().is_terminal()
        && io::stdout().is_terminal()
        && orte.len() > 1
    {
        for (i, ort) in orte.iter().enumerate() {
            eprintln!("{}. {}", i + 1, ort.beschreibung());
        }
        eprint!("Ort wählen [1]: ");
        io::stderr().flush()?;
        let mut text = String::new();
        io::stdin().read_line(&mut text)?;
        if text.trim().is_empty() {
            0
        } else {
            text.trim()
                .parse::<usize>()?
                .checked_sub(1)
                .ok_or("Die Auswahl beginnt bei 1.")?
        }
    } else {
        0
    };
    let ort = orte
        .get(index)
        .ok_or("Diese Ortsnummer wurde nicht gefunden.")?
        .clone();
    let wetter = api::wetter(&client, &ort)?;
    let bericht = Bericht { ort, wetter };
    match modus {
        Modus::Json => println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version":1,"location":bericht.ort,"weather":bericht.wetter,
                "units":{"temperature":"celsius","wind":"km/h","precipitation":"mm","probability":"percent","pressure":"hPa","visibility":"m","duration":"seconds"},
                "source":{"weather":"https://open-meteo.com/","locations":"GeoNames","license":"CC BY 4.0"}
            }))?
        ),
        Modus::Plain => print!("{}", ascii(&plain(&bericht))),
        _ => einmal(bericht)?,
    }
    Ok(())
}

fn argumente(args: &[String]) -> Result<(Modus, Option<String>, Option<usize>), String> {
    let mut modus = Modus::Einmal;
    let mut name = Vec::new();
    let mut pick = None;
    let mut i = 0;
    let mut nur_ort = false;
    while i < args.len() {
        let arg = &args[i];
        if nur_ort {
            name.push(arg.clone());
            i += 1;
            continue;
        }
        match arg.as_str() {
            "--" => nur_ort = true,
            "--tui" | "--plain" | "--json" => {
                if modus != Modus::Einmal {
                    return Err("Bitte nur einen Ausgabemodus wählen.".into());
                }
                modus = match arg.as_str() {
                    "--tui" => Modus::Tui,
                    "--plain" => Modus::Plain,
                    _ => Modus::Json,
                };
            }
            "--pick" => {
                i += 1;
                let n = args
                    .get(i)
                    .and_then(|s| s.parse::<usize>().ok())
                    .filter(|n| *n > 0)
                    .ok_or("--pick benötigt eine positive Zahl.")?;
                pick = Some(n);
            }
            _ if arg.starts_with('-') => {
                return Err(format!("Unbekannte Option: {arg}. Hilfe: --help"));
            }
            _ => name.push(arg.clone()),
        }
        i += 1;
    }
    Ok((modus, (!name.is_empty()).then(|| name.join(" ")), pick))
}

fn plain(b: &Bericht) -> String {
    use format::*;
    let w = &b.wetter;
    let a = &w.current;
    let mut text = format!(
        "WETTER / {}\nStand: {} ({})\n\nJetzt: {} - {}\nGefuehlt: {}\n{}\n\nWind: {} aus {}, Boeen {}\nNiederschlag heute: {}, Risiko max. {}\nSonnenuntergang: {}\n\nTAGESPROGNOSE\n",
        b.ort.kurz(),
        a.time.replace('T', " "),
        w.timezone,
        wert(a.temperature_2m, " C"),
        wettertext(a.weather_code),
        wert(a.apparent_temperature, " C"),
        ui::zusammenfassung(w),
        wert(a.wind_speed_10m, " km/h"),
        windrichtung(a.wind_direction_10m),
        wert(a.wind_gusts_10m, " km/h"),
        wert(w.tageswert(&w.daily.precipitation_sum), " mm"),
        ganz(w.tageswert(&w.daily.precipitation_probability_max), " %"),
        uhrzeit(w.tageszeit(&w.daily.sunset))
    );
    for (i, tag) in w.daily.time.iter().enumerate().take(3) {
        text.push_str(&format!(
            "{}: {} bis {} | {} | Risiko max. {}\n",
            tag,
            wert(w.daily.temperature_2m_min.get(i).copied().flatten(), " C"),
            wert(w.daily.temperature_2m_max.get(i).copied().flatten(), " C"),
            wettertext(w.daily.weather_code.get(i).copied().flatten()),
            ganz(
                w.daily
                    .precipitation_probability_max
                    .get(i)
                    .copied()
                    .flatten(),
                " %"
            )
        ));
    }
    text.push_str("\nOpen-Meteo: Modelldaten, CC BY 4.0. Orte: GeoNames.\n");
    text
}
fn ascii(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'ä' => "ae".into(),
            'ö' => "oe".into(),
            'ü' => "ue".into(),
            'Ä' => "Ae".into(),
            'Ö' => "Oe".into(),
            'Ü' => "Ue".into(),
            'ß' => "ss".into(),
            '–' | '—' => "--".into(),
            '°' => " deg".into(),
            '·' => "/".into(),
            c if c.is_ascii() => c.to_string(),
            _ => "?".into(),
        })
        .collect()
}
fn einmal(bericht: Bericht) -> Ergebnis<()> {
    let width = if io::stdout().is_terminal() {
        crossterm::terminal::size().map_or(96, |(w, _)| w.clamp(40, 140))
    } else {
        96
    };
    let height = if width < 80 { 34 } else { 38 };
    let app = app::App::neu(Some(bericht), false);
    let mut terminal = Terminal::new(TestBackend::new(width, height))?;
    terminal.draw(|f| ui::zeichnen(f, &app))?;
    let colors = io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
    let mut out = io::stdout().lock();
    let buffer = terminal.backend().buffer();
    for y in 0..height {
        let mut x = 0;
        while x < width {
            let cell = &buffer[(x, y)];
            if colors {
                write!(out, "{}{}", ansi(cell.fg, false), ansi(cell.bg, true))?;
            }
            write!(out, "{}", cell.symbol())?;
            x += cell.symbol().width().max(1) as u16;
        }
        if colors {
            write!(out, "\x1b[0m")?;
        }
        writeln!(out)?;
    }
    Ok(())
}
fn ansi(c: Color, bg: bool) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("\x1b[{};2;{r};{g};{b}m", if bg { 48 } else { 38 }),
        _ => format!("\x1b[{}m", if bg { 49 } else { 39 }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cli_modes_and_ascii() {
        assert_eq!(
            argumente(&["--json".into(), "New York".into()]).unwrap().0,
            Modus::Json
        );
        assert!(argumente(&["--json".into(), "--tui".into()]).is_err());
        assert!(argumente(&["--pick".into(), "0".into()]).is_err());
        assert_eq!(ascii("Zürich · —"), "Zuerich / --");
    }
}
