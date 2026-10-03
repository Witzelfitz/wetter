use crate::{
    app::{App, Dialog},
    data::*,
    format::*,
    theme::Theme,
};
use chrono::NaiveDateTime;
use ratatui::{prelude::*, widgets::*};
use tui_big_text::{BigText, PixelSize};
use unicode_width::UnicodeWidthStr;

pub fn zeichnen(f: &mut Frame, app: &App) {
    let t = Theme::neu();
    f.render_widget(Block::default().style(t.basis()), f.area());
    let mut area = f.area().inner(Margin::new(2, 1));
    let breite = area.width.min(132);
    area.x += (area.width - breite) / 2;
    area.width = breite;
    if area.width < 28 || area.height < 10 {
        f.render_widget(
            Paragraph::new("Fenster vergrössern\n(mind. 32 × 12)\nq: Beenden").style(t.basis()),
            area,
        );
        return;
    }
    let parts = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Min(1),
        Constraint::Length(3),
    ])
    .split(area);
    let ort = app
        .bericht
        .as_ref()
        .map_or("Dein Wetter".to_owned(), |b| b.ort.kurz());
    let zeit = app.bericht.as_ref().map_or(String::new(), |b| {
        format!(
            "{} · {}",
            b.wetter.current.time.replace('T', " "),
            b.wetter.timezone
        )
    });
    let header = if area.width >= 85 {
        format!("WETTER / {ort}   ·   {zeit}")
    } else {
        format!("WETTER / {ort}\n{zeit}")
    };
    f.render_widget(Paragraph::new(header).style(t.basis()), parts[0]);
    f.render_widget(
        Tabs::new(["Übersicht", "Stunden", "Details"])
            .select(app.ansicht)
            .style(Style::default().fg(t.muted))
            .highlight_style(Style::default().fg(t.active).bold())
            .divider("   "),
        parts[1],
    );
    if let Some(b) = &app.bericht {
        match app.ansicht {
            1 => stunden(f, parts[2], b, app, t),
            2 => details(f, parts[2], b, app.offset, t),
            _ => uebersicht(f, parts[2], b, t),
        }
    } else {
        f.render_widget(
            Paragraph::new(
                "\nWie ist es draussen?\n\nOrt suchen mit /\n\nWetterdaten von Open-Meteo.",
            )
            .style(t.basis()),
            parts[2],
        );
    }
    let status = if app.status.is_empty() {
        "Open-Meteo · Modelldaten · CC BY 4.0 · Orte: GeoNames"
    } else {
        &app.status
    };
    let keys = if !app.interaktiv {
        "Einmalige Ausgabe · wetter --tui für die interaktive Ansicht"
    } else if area.width < 75 {
        "[Tab] Ansicht [/] Ort [r] [q] Ende"
    } else if app.ansicht == 1 {
        "[← →] Stunde  [Tab] Ansicht  [/] Ort  [r] Neu  [?] Hilfe  [q] Ende"
    } else {
        "[← →] Ansicht  [/] Ort  [r] Neu laden  [?] Hilfe  [q] Ende"
    };
    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                "─".repeat(parts[3].width as usize),
                Style::default().fg(t.border),
            )),
            Line::from(Span::styled(keys, Style::default().fg(t.muted))),
            Line::from(Span::styled(
                status,
                Style::default().fg(if app.fehler {
                    t.sun
                } else if app.laedt {
                    t.active
                } else {
                    t.muted
                }),
            )),
        ]),
        parts[3],
    );
    if let Some(dialog) = &app.dialog {
        overlay(f, area, dialog, t);
    }
}

fn uebersicht(f: &mut Frame, area: Rect, b: &Bericht, t: Theme) {
    // Zusätzlicher Platz bleibt ausserhalb der Informationsgruppen.
    let area = Rect {
        height: area.height.min(30),
        ..area
    };
    if area.height < 24 {
        let mit_kurve = area.height >= 20 && area.width >= 54;
        let mit_ausblick = area.height >= 14;
        let hero_h = if mit_kurve { 3 } else { 4.min(area.height / 3) };
        hero(
            f,
            Rect {
                height: hero_h,
                ..area
            },
            b,
            t,
        );
        let mut y = area.y + hero_h;
        if mit_kurve {
            kurve(
                f,
                Rect {
                    y,
                    height: 9,
                    ..area
                },
                &b.wetter,
                12,
                None,
                t,
            );
            y += 9;
        } else {
            f.render_widget(
                Paragraph::new("Stundenkurve: Tab → Stunden").style(Style::default().fg(t.muted)),
                Rect {
                    y,
                    height: 1,
                    ..area
                },
            );
            y += 1;
        }
        let w = &b.wetter;
        let metrics = Rect {
            y,
            height: 3,
            ..area
        };
        if area.width >= 54 {
            kennzahlen(f, metrics, b, t);
        } else {
            let text = format!(
                "Wind {} · Böen {}\nHeute {} · Risiko {}\nSonnenuntergang {}",
                wert(w.current.wind_speed_10m, " km/h"),
                wert(w.current.wind_gusts_10m, ""),
                wert(w.tageswert(&w.daily.precipitation_sum), " mm"),
                ganz(w.tageswert(&w.daily.precipitation_probability_max), " %"),
                uhrzeit(w.tageszeit(&w.daily.sunset))
            );
            f.render_widget(Paragraph::new(text).style(t.basis()), metrics);
        }
        y += 3;
        if mit_ausblick {
            ausblick(
                f,
                Rect {
                    y,
                    height: area.bottom().saturating_sub(y),
                    ..area
                },
                b,
                t,
            );
        }
        return;
    }
    if area.width >= 106 && area.height >= 23 {
        let rows = Layout::vertical([
            Constraint::Length(14.min(area.height.saturating_sub(11))),
            Constraint::Length(5),
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .split(area);
        let cols = Layout::horizontal([Constraint::Length(44), Constraint::Min(50)])
            .spacing(3)
            .split(rows[0]);
        hero(f, cols[0], b, t);
        kurve(f, cols[1], &b.wetter, 12, None, t);
        kennzahlen(f, rows[1], b, t);
        ausblick(f, rows[2], b, t);
    } else {
        let gross = area.width >= 76 && area.height >= 26;
        let rows = Layout::vertical([
            Constraint::Length(if gross { 7 } else { 4 }),
            Constraint::Length(
                area.height
                    .saturating_sub(if gross { 17 } else { 14 })
                    .min(12),
            ),
            Constraint::Length(if area.width < 54 { 7 } else { 4 }),
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .split(area);
        hero(f, rows[0], b, t);
        if rows[1].height >= 6 {
            kurve(f, rows[1], &b.wetter, 12, None, t);
        } else {
            f.render_widget(
                Paragraph::new("Stundenverlauf in der Ansicht »Stunden«")
                    .style(Style::default().fg(t.muted)),
                rows[1],
            );
        }
        kennzahlen(f, rows[2], b, t);
        ausblick(f, rows[3], b, t);
    }
}

fn hero(f: &mut Frame, area: Rect, b: &Bericht, t: Theme) {
    let w = &b.wetter;
    let a = &w.current;
    let infos = vec![
        Line::from(Span::styled(
            wettertext(a.weather_code),
            Style::default().fg(t.text).bold(),
        )),
        Line::from(format!("Gefühlt {}", wert(a.apparent_temperature, " °C"))),
        Line::from(format!(
            "Heute {} bis {}",
            wert(w.tageswert(&w.daily.temperature_2m_min), "°"),
            wert(w.tageswert(&w.daily.temperature_2m_max), "°")
        )),
    ];
    if area.width >= 40 && area.height >= 7 {
        let wide = area.width >= 76;
        let upper = Rect { height: 5, ..area };
        let widths = if wide {
            vec![
                Constraint::Length(20),
                Constraint::Length(28),
                Constraint::Min(20),
            ]
        } else {
            vec![Constraint::Length(18), Constraint::Min(20)]
        };
        let cols = Layout::horizontal(widths).split(upper);
        let icon = wetterbild(a.weather_code, a.is_day == Some(0)).0;
        // Jedes Wettersymbol belegt dasselbe Raster von 19 × 5 Zeichen.
        f.render_widget(
            Paragraph::new(icon.join("\n")).style(Style::default().fg(if a.is_day == Some(0) {
                t.rain
            } else {
                t.sun
            })),
            cols[0],
        );
        let zahl = wert(a.temperature_2m, "");
        f.render_widget(
            BigText::builder()
                .pixel_size(PixelSize::Quadrant)
                .style(Style::default().fg(t.text))
                .lines(vec![format!("{zahl}°").into()])
                .build(),
            cols[1],
        );
        if wide {
            f.render_widget(
                Paragraph::new(infos).style(Style::default().fg(t.muted)),
                cols[2],
            );
        } else {
            f.render_widget(
                Paragraph::new(infos).style(Style::default().fg(t.muted)),
                Rect {
                    x: area.x,
                    y: area.y + 5,
                    width: area.width,
                    height: 3.min(area.height - 5),
                },
            );
        }
        let y = if wide { area.y + 5 } else { area.y + 9 };
        if y < area.bottom() {
            f.render_widget(
                Paragraph::new(zusammenfassung(w))
                    .wrap(Wrap { trim: true })
                    .style(Style::default().fg(t.muted)),
                Rect {
                    x: area.x,
                    y,
                    width: area.width,
                    height: area.bottom() - y,
                },
            );
        }
    } else {
        let mut lines = vec![Line::from(vec![
            Span::styled(
                wert(a.temperature_2m, " °C"),
                Style::default().fg(t.text).bold(),
            ),
            Span::raw(format!("  {}", wettertext(a.weather_code))),
        ])];
        lines.extend(infos.into_iter().skip(1));
        lines.push(Line::from(zusammenfassung(w)));
        f.render_widget(Paragraph::new(lines).style(t.basis()), area);
    }
}

pub fn zusammenfassung(w: &Wetter) -> String {
    let mut teile = Vec::new();
    if let Some(temp) = w.current.temperature_2m {
        teile.push(
            if temp < 5.0 {
                "Kalt"
            } else if temp < 15.0 {
                "Frisch"
            } else if temp < 25.0 {
                "Mild"
            } else {
                "Warm"
            }
            .to_owned(),
        );
    }
    if let Some(wind) = w.current.wind_speed_10m {
        teile.push(
            if wind < 10.0 {
                "wenig Wind"
            } else if wind < 25.0 {
                "windig"
            } else {
                "kräftiger Wind"
            }
            .to_owned(),
        );
    }
    let mut text = teile.join(", ");
    if !text.is_empty() {
        text.push_str(". ");
    }
    text.push_str(&sonnenstatus(w));
    text
}

pub fn sonnenstatus(w: &Wetter) -> String {
    let parse = |s: &str| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").ok();
    let now = parse(&w.current.time);
    let set = w.tageszeit(&w.daily.sunset).and_then(parse);
    match (now, set) {
        (Some(now), Some(set)) if set > now => format!(
            "Sonnenuntergang in {}.",
            dauer(Some((set - now).num_seconds() as f64))
        ),
        (Some(_), Some(_)) => "Sonne ist bereits untergegangen.".to_owned(),
        _ => "Sonnenuntergang nicht verfügbar.".to_owned(),
    }
}

fn kennzahlen(f: &mut Frame, area: Rect, b: &Bericht, t: Theme) {
    let w = &b.wetter;
    let a = &w.current;
    let values = [
        (
            "WIND JETZT",
            wert(a.wind_speed_10m, " km/h"),
            format!(
                "aus {} · Böen {}",
                windrichtung(a.wind_direction_10m),
                wert(a.wind_gusts_10m, "")
            ),
        ),
        (
            "NIEDERSCHLAG HEUTE",
            wert(w.tageswert(&w.daily.precipitation_sum), " mm"),
            format!(
                "Risiko max. {}",
                ganz(w.tageswert(&w.daily.precipitation_probability_max), " %")
            ),
        ),
        (
            "SONNENUNTERGANG",
            uhrzeit(w.tageszeit(&w.daily.sunset)).to_owned(),
            sonnenstatus(w),
        ),
    ];
    if area.width < 54 {
        let lines: Vec<Line> = values
            .into_iter()
            .flat_map(|(label, value, extra)| {
                vec![
                    Line::from(format!("{label}: {value}")),
                    Line::from(Span::styled(extra, Style::default().fg(t.muted))),
                ]
            })
            .collect();
        f.render_widget(Paragraph::new(lines), area);
    } else {
        let cols = Layout::horizontal([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .spacing(2)
        .split(area);
        for (i, (label, value, extra)) in values.into_iter().enumerate() {
            let extra = if i == 2 {
                extra
                    .replace("Sonnenuntergang in ", "Noch ")
                    .trim_end_matches('.')
                    .to_owned()
            } else {
                extra
            };
            f.render_widget(
                Paragraph::new(vec![
                    Line::from(Span::styled(label, Style::default().fg(t.muted))),
                    Line::from(Span::styled(value, Style::default().fg(t.text).bold())),
                    Line::from(Span::styled(extra, Style::default().fg(t.muted))),
                ]),
                cols[i],
            );
        }
    }
}

fn ausblick(f: &mut Frame, area: Rect, b: &Bericht, t: Theme) {
    let area = Rect {
        width: area.width.min(78),
        ..area
    };
    let w = &b.wetter;
    let rows = w.daily.time.iter().enumerate().take(3).map(|(i, date)| {
        let name = if Some(i) == w.heute() {
            "Heute".into()
        } else if Some(i) == w.heute().map(|n| n + 1) {
            "Morgen".into()
        } else {
            datum_kurz(date)
        };
        Row::new(vec![
            name,
            wettertext(w.daily.weather_code.get(i).copied().flatten()).to_owned(),
            wert(w.daily.temperature_2m_min.get(i).copied().flatten(), "°"),
            wert(w.daily.temperature_2m_max.get(i).copied().flatten(), "°"),
            ganz(
                w.daily
                    .precipitation_probability_max
                    .get(i)
                    .copied()
                    .flatten(),
                " %",
            ),
        ])
    });
    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(area.width.saturating_sub(8 + 7 + 7 + 11 + 4).min(26)),
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Length(if area.width < 65 { 7 } else { 11 }),
        ],
    )
    .header(
        Row::new([
            "Tag",
            "Tageswetter",
            "Min.",
            "Max.",
            if area.width < 65 {
                "Risiko"
            } else {
                "Risiko max."
            },
        ])
        .style(Style::default().fg(t.muted)),
    )
    .block(
        Block::new()
            .title("AUSBLICK · TAGESPROGNOSE")
            .title_style(Style::default().fg(t.muted)),
    )
    .column_spacing(1);
    f.render_widget(table, area);
}

// Lücken bleiben getrennte Segmente. Ein fehlender Wert ist niemals 0 Grad.
pub fn temperatursegmente(w: &Wetter, indices: &[usize]) -> Vec<Vec<(f64, f64)>> {
    let mut result = Vec::new();
    let mut segment = Vec::new();
    for (x, &i) in indices.iter().enumerate() {
        match w
            .hourly
            .temperature_2m
            .get(i)
            .copied()
            .flatten()
            .filter(|v| v.is_finite())
        {
            Some(v) => segment.push((x as f64, v)),
            None => {
                if !segment.is_empty() {
                    result.push(std::mem::take(&mut segment));
                }
            }
        }
    }
    if !segment.is_empty() {
        result.push(segment);
    }
    result
}

// Gleiche Braille-Rasterprojektion wie Ratatui: zwei horizontale Punkte pro Zelle.
fn stunden_spalte(stunde: usize, horizont: usize, breite: u16) -> u16 {
    (((stunde as f64 / horizont.max(1) as f64) * (f64::from(breite) * 2.0 - 1.0).max(0.0)).round()
        as u16
        / 2)
    .min(breite.saturating_sub(1))
}

fn kurve(f: &mut Frame, area: Rect, w: &Wetter, horizont: usize, auswahl: Option<usize>, t: Theme) {
    if area.height < 7 || area.width < 18 {
        return;
    }
    f.render_widget(Block::default().style(t.basis()), area);
    let indices: Vec<_> = w.stunden_indices().into_iter().take(horizont + 1).collect();
    let segments = temperatursegmente(w, &indices);
    let bounds: Vec<f64> = segments.iter().flatten().map(|(_, y)| *y).collect();
    let (low, high) = if bounds.is_empty() {
        (0.0, 1.0)
    } else {
        (
            bounds.iter().copied().fold(f64::INFINITY, f64::min).floor() - 1.0,
            bounds
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max)
                .ceil()
                + 1.0,
        )
    };
    let labels = [
        format!("{high:.0}°"),
        format!("{:.0}°", (low + high) / 2.0),
        format!("{low:.0}°"),
    ];
    let rand = if bounds.is_empty() {
        5
    } else {
        labels.iter().map(|s| s.width() as u16).max().unwrap_or(3) + 2
    };
    let plot = Rect::new(
        area.x + rand,
        area.y + 1,
        area.width.saturating_sub(rand),
        area.height - 5,
    );
    let start = indices
        .first()
        .and_then(|&i| w.hourly.time.get(i))
        .map_or("–", |s| uhrzeit(Some(s)));
    f.render_widget(
        Paragraph::new(format!("TEMPERATUR · {horizont} H AB {start}"))
            .style(t.basis().fg(t.muted)),
        Rect { height: 1, ..area },
    );
    let sets = segments
        .iter()
        .map(|s| {
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .graph_type(if s.len() == 1 {
                    GraphType::Scatter
                } else {
                    GraphType::Line
                })
                .style(Style::default().fg(t.sun))
                .data(s)
        })
        .collect::<Vec<_>>();
    // Achsenbeschriftung ausserhalb des Charts: Temperatur und Regen nutzen exakt
    // dieselbe Plot-Breite, auch bei negativen Temperaturen und beim Resize.
    f.render_widget(
        Chart::new(sets)
            .style(t.basis())
            .block(Block::default().style(t.basis()))
            .x_axis(Axis::default().bounds([0.0, horizont as f64]))
            .y_axis(Axis::default().bounds([low, high])),
        plot,
    );
    if bounds.is_empty() {
        f.render_widget(
            Paragraph::new("Keine Temperaturdaten verfügbar.")
                .style(t.basis().fg(t.muted))
                .wrap(Wrap { trim: true }),
            plot,
        );
    } else {
        for (n, label) in labels.into_iter().enumerate() {
            let y = plot.y + n as u16 * (plot.height - 1) / 2;
            f.render_widget(
                Paragraph::new(label)
                    .alignment(Alignment::Right)
                    .style(t.basis().fg(t.muted)),
                Rect::new(area.x, y, rand - 2, 1),
            );
        }
    }
    let zeit_y = plot.bottom();
    let regen_y = zeit_y + 2;
    let schritt = if plot.width >= 60 {
        horizont / 4
    } else {
        horizont / 2
    };
    for n in (0..=horizont).step_by(schritt.max(1)) {
        let label = indices
            .get(n)
            .and_then(|&i| w.hourly.time.get(i))
            .map_or_else(|| format!("+{n} h"), |s| uhrzeit(Some(s)).to_owned());
        let x = (plot.x + stunden_spalte(n, horizont, plot.width))
            .saturating_sub((label.width() / 2) as u16)
            .clamp(plot.x, plot.right().saturating_sub(label.width() as u16));
        f.render_widget(
            Paragraph::new(label).style(t.basis().fg(t.muted)),
            Rect::new(x, zeit_y, 5.min(plot.right() - x), 1),
        );
    }
    f.render_widget(
        Paragraph::new("Regenrisiko · %").style(t.basis().fg(t.muted)),
        Rect::new(area.x, zeit_y + 1, area.width, 1),
    );
    // Ein Zeichen je echter Stunde, auf derselben Zeitachse wie die Temperatur.
    // Fehlende Stunden bleiben erkennbar; es werden keine Werte interpoliert.
    for n in 0..=horizont {
        let chance = indices
            .get(n)
            .and_then(|&i| w.hourly.precipitation_probability.get(i))
            .copied()
            .flatten()
            .filter(|v| v.is_finite());
        let (symbol, color) = match chance {
            Some(0.0) => ("─", t.border),
            Some(v) => (
                ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"]
                    [((v.clamp(0.0, 100.0) / 100.0) * 7.0).round() as usize],
                t.rain,
            ),
            None => ("·", t.muted),
        };
        f.buffer_mut()[(plot.x + stunden_spalte(n, horizont, plot.width), regen_y)]
            .set_symbol(symbol)
            .set_style(t.basis().fg(color));
    }
    if let Some(index) = auswahl.filter(|&i| i < indices.len()) {
        let x = plot.x + stunden_spalte(index, horizont, plot.width);
        for y in plot.y..=regen_y {
            let cell = &mut f.buffer_mut()[(x, y)];
            if cell.symbol() == " " {
                cell.set_symbol("┆");
            }
            cell.set_fg(t.active);
        }
    }
    let komplett = indices.len() == horizont + 1;
    let regen_vollstaendig = komplett
        && indices.iter().all(|&i| {
            w.hourly
                .precipitation_probability
                .get(i)
                .copied()
                .flatten()
                .is_some_and(f64::is_finite)
        });
    let trocken = regen_vollstaendig
        && indices.iter().all(|&i| {
            w.hourly.precipitation_probability.get(i) == Some(&Some(0.0))
                && w.hourly.precipitation.get(i) == Some(&Some(0.0))
        });
    let hinweis = if trocken {
        if area.width >= 58 {
            "Für diesen Zeitraum kein Niederschlag prognostiziert."
        } else {
            "Kein Niederschlag prognostiziert."
        }
    } else if !regen_vollstaendig || bounds.len() != horizont + 1 {
        "Lücken / · = fehlende Stundendaten"
    } else {
        ""
    };
    f.render_widget(
        Paragraph::new(hinweis).style(t.basis().fg(t.muted)),
        Rect::new(area.x, regen_y + 1, area.width, 1),
    );
}

fn stunden(f: &mut Frame, area: Rect, b: &Bericht, app: &App, t: Theme) {
    let hoehe = if area.height >= 13 {
        area.height.saturating_sub(6).min(16)
    } else {
        0
    };
    let parts = Layout::vertical([
        Constraint::Length(hoehe),
        Constraint::Length(6),
        Constraint::Min(0),
    ])
    .split(area);
    kurve(f, parts[0], &b.wetter, 24, Some(app.stunde), t);
    let w = &b.wetter;
    if let Some(&i) = w.stunden_indices().get(app.stunde) {
        let zeit = w
            .hourly
            .time
            .get(i)
            .map_or("–", String::as_str)
            .replace('T', " ");
        let text = vec![
            Line::from(Span::styled(
                format!(
                    "{zeit} · {}",
                    wettertext(w.hourly.weather_code.get(i).copied().flatten())
                ),
                Style::default().fg(t.active),
            )),
            Line::from(vec![
                Span::styled(
                    wert(w.hourly.temperature_2m.get(i).copied().flatten(), " °C"),
                    Style::default().bold(),
                ),
                Span::raw(format!(
                    "    Wind {}",
                    wert(w.hourly.wind_speed_10m.get(i).copied().flatten(), " km/h")
                )),
            ]),
            Line::from(format!(
                "Regenrisiko {} · Niederschlag {}",
                ganz(
                    w.hourly.precipitation_probability.get(i).copied().flatten(),
                    " %"
                ),
                wert(w.hourly.precipitation.get(i).copied().flatten(), " mm")
            )),
            Line::from(Span::styled(
                "Regenmenge: vorangehende Stunde · Ortszeit",
                Style::default().fg(t.muted),
            )),
        ];
        f.render_widget(
            Paragraph::new(text)
                .wrap(Wrap { trim: true })
                .style(t.basis()),
            parts[1],
        );
    }
}

fn detail_gruppe(
    buf: &mut Buffer,
    area: Rect,
    titel: &str,
    wert: String,
    label: &str,
    zeilen: Vec<Line<'static>>,
    t: Theme,
) {
    let mut lines = vec![
        Line::from(Span::styled(titel.to_owned(), Style::default().fg(t.muted))),
        Line::default(),
        Line::from(Span::styled(wert, Style::default().fg(t.text).bold())),
        Line::from(Span::styled(label.to_owned(), Style::default().fg(t.muted))),
        Line::default(),
    ];
    lines.extend(zeilen);
    Paragraph::new(lines).style(t.basis()).render(area, buf);
}

fn tageslicht(w: &Wetter, breite: u16, t: Theme) -> Line<'static> {
    let parse = |s: &str| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").ok();
    let zeiten = w
        .tageszeit(&w.daily.sunrise)
        .and_then(parse)
        .zip(w.tageszeit(&w.daily.sunset).and_then(parse))
        .zip(parse(&w.current.time));
    let Some(((auf, unter), jetzt)) = zeiten.filter(|((auf, unter), jetzt)| {
        auf < unter && auf.date() == jetzt.date() && unter.date() == jetzt.date()
    }) else {
        return Line::from(Span::styled(
            "— Tageslichtverlauf nicht verfügbar",
            Style::default().fg(t.muted),
        ));
    };
    let null = jetzt.date().and_hms_opt(0, 0, 0).unwrap();
    let position = |zeit: NaiveDateTime| {
        (((zeit - null).num_minutes() as f64 / 1440.0) * f64::from(breite.saturating_sub(1)))
            .round() as u16
    };
    Line::from(
        (0..breite)
            .map(|x| {
                let (symbol, farbe) = if x == position(jetzt) {
                    ("│", t.active)
                } else if (position(auf)..=position(unter)).contains(&x) {
                    ("━", t.sun)
                } else {
                    ("─", t.border)
                };
                Span::styled(symbol, Style::default().fg(farbe))
            })
            .collect::<Vec<_>>(),
    )
}

fn details(f: &mut Frame, area: Rect, b: &Bericht, offset: u16, t: Theme) {
    if area.is_empty() {
        return;
    }
    let w = &b.wetter;
    let a = &w.current;
    let zwei = area.width >= 76;
    let breite = if zwei {
        (area.width - 4) / 2
    } else {
        area.width
    };
    let hoehe = if zwei { 26 } else { 44 };
    let virtuell = Rect::new(0, 0, area.width, hoehe);
    let mut buf = Buffer::empty(virtuell);
    buf.set_style(virtuell, t.basis());
    let luft = Rect::new(0, 0, breite, 9);
    let wind = Rect::new(
        if zwei { breite + 4 } else { 0 },
        if zwei { 0 } else { 9 },
        breite,
        9,
    );
    let sonne = Rect::new(0, if zwei { 10 } else { 18 }, breite, 14);
    let weitere = Rect::new(
        if zwei { breite + 4 } else { 0 },
        if zwei { 10 } else { 33 },
        breite,
        9,
    );
    detail_gruppe(
        &mut buf,
        luft,
        "LUFT · JETZT",
        ganz(a.relative_humidity_2m, " %"),
        "Luftfeuchte",
        vec![
            Line::from(format!("Taupunkt     {}", wert(a.dew_point_2m, " °C"))),
            Line::from(format!("Bewölkung    {}", ganz(a.cloud_cover, " %"))),
        ],
        t,
    );
    detail_gruppe(
        &mut buf,
        wind,
        "WIND · JETZT",
        wert(a.wind_speed_10m, " km/h"),
        "Windgeschwindigkeit",
        vec![
            Line::from(format!(
                "Richtung     aus {}",
                windrichtung(a.wind_direction_10m)
            )),
            Line::from(format!("Böen         {}", wert(a.wind_gusts_10m, " km/h"))),
        ],
        t,
    );
    let skala = format!(
        "00{: ^width$}24",
        "12",
        width = breite.saturating_sub(4) as usize
    );
    detail_gruppe(
        &mut buf,
        sonne,
        "SONNE · HEUTE / PROGNOSE",
        dauer(w.tageswert(&w.daily.daylight_duration)),
        "Tageslicht",
        vec![
            Line::from(format!(
                "Auf {} · Unter {}",
                uhrzeit(w.tageszeit(&w.daily.sunrise)),
                uhrzeit(w.tageszeit(&w.daily.sunset))
            )),
            tageslicht(w, breite, t),
            Line::from(Span::styled(skala, Style::default().fg(t.muted))),
            Line::default(),
            Line::from(format!(
                "Sonnenschein  {}",
                dauer(w.tageswert(&w.daily.sunshine_duration))
            )),
            Line::from(format!(
                "UV-Maximum    {}",
                wert(w.tageswert(&w.daily.uv_index_max), "")
            )),
            Line::from(format!(
                "Regen heute   {}",
                wert(w.tageswert(&w.daily.precipitation_sum), " mm")
            )),
        ],
        t,
    );
    detail_gruppe(
        &mut buf,
        weitere,
        "WEITERE WERTE · JETZT",
        wert(a.visibility.map(|m| m / 1000.0), " km"),
        "Sichtweite",
        vec![
            Line::from(format!("Luftdruck    {}", ganz(a.pressure_msl, " hPa"))),
            Line::from(Span::styled(
                "bezogen auf Meereshöhe",
                Style::default().fg(t.muted),
            )),
        ],
        t,
    );
    Paragraph::new(format!("Ortszeit · {} · ↑ ↓ Scrollen", w.timezone))
        .style(t.basis().fg(t.muted))
        .render(Rect::new(0, hoehe - 1, area.width, 1), &mut buf);
    let scroll = offset.min(hoehe.saturating_sub(area.height));
    for y in 0..area.height.min(hoehe - scroll) {
        for x in 0..area.width {
            f.buffer_mut()[(area.x + x, area.y + y)] = buf[(x, y + scroll)].clone();
        }
    }
}

fn overlay(f: &mut Frame, area: Rect, dialog: &Dialog, t: Theme) {
    let width = area.width.min(76);
    let height = match dialog {
        Dialog::Suche(_) => 7,
        Dialog::Orte(_, _) => 18,
        Dialog::Hilfe => 16,
    }
    .min(area.height);
    let popup = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    f.render_widget(Clear, popup);
    let title = match dialog {
        Dialog::Suche(_) => " ORT SUCHEN ",
        Dialog::Orte(_, _) => " WELCHEN ORT MEINST DU? ",
        Dialog::Hilfe => " TASTEN & DATEN ",
    };
    let block = Block::bordered()
        .title(title)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(t.active))
        .style(t.basis().bg(t.surface));
    let inner = block.inner(popup);
    f.render_widget(block, popup);
    match dialog {
        Dialog::Suche(input)=>{
            let verfuegbar=inner.width.saturating_sub(3) as usize;
            let chars:Vec<_>=input.text.chars().collect();
            let mut start=input.cursor;
            let mut cursor_width=0;
            while start>0 {let w=chars[start-1].to_string().width();if cursor_width+w>=verfuegbar{break;}start-=1;cursor_width+=w;}
            let text:String=chars[start..].iter().collect();
            f.render_widget(Paragraph::new(format!("Ort oder Postleitzahl\n\n> {text}\n\nEnter: suchen · Esc: zurück")).style(t.basis().bg(t.surface)),inner);
            f.set_cursor_position((inner.x+2+cursor_width as u16,inner.y+2));
        }
        Dialog::Orte(orte,index)=>{
            let items:Vec<_>=orte.iter().map(|ort|ListItem::new(ort.beschreibung())).collect();
            let mut state=ListState::default().with_selected(Some(*index));
            f.render_stateful_widget(List::new(items).highlight_symbol("› ").highlight_style(Style::default().fg(t.active).bold()).block(Block::new().title_bottom("↑ ↓ wählen · Enter öffnen · Esc zurück")),inner,&mut state);
        }
        Dialog::Hilfe=>f.render_widget(Paragraph::new("Tab / 1 2 3   Übersicht, Stunden, Details\n← →           Ansicht; in Stunden: Stunde wählen\n↑ ↓           Details scrollen / Ort wählen\n/             Neuen Ort suchen\nr             Wetter aktualisieren\nq / Esc       Beenden (im Dialog: zurück)\nCtrl+C        Sofort beenden\n\nGelb: Temperatur · Blau: Niederschlagsrisiko\nTürkis: Auswahl / Modellzeit · — / ·: fehlende Daten\nZeiten am gewählten Ort, Stand laut Wettermodell.\n\nBeliebige Taste: zurück").style(t.basis().bg(t.surface)).wrap(Wrap{trim:true}),inner),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn text(buf: &Buffer) -> String {
        (0..buf.area.height)
            .map(|y| {
                (0..buf.area.width)
                    .map(|x| buf[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn temperatur_regen_und_auswahl_teilen_dieselbe_stundenspalte() {
        let t = Theme {
            bg: Color::Rgb(16, 21, 29),
            sun: Color::Yellow,
            rain: Color::Blue,
            active: Color::Cyan,
            ..Theme::neu()
        };
        for width in [33, 65, 110, 132] {
            for stunde in [0, 5, 12, 24] {
                let mut b = crate::test_support::beispiel();
                b.wetter.hourly.temperature_2m = vec![None; 25];
                b.wetter.hourly.temperature_2m[stunde] = Some(-12.0);
                b.wetter.hourly.precipitation_probability = vec![None; 25];
                b.wetter.hourly.precipitation_probability[stunde] = Some(80.0);
                let mut terminal = Terminal::new(TestBackend::new(width, 16)).unwrap();
                terminal
                    .draw(|f| kurve(f, f.area(), &b.wetter, 24, None, t))
                    .unwrap();
                let buf = terminal.backend().buffer();
                let punkt = (1..12)
                    .find_map(|y| (0..width).find(|&x| buf[(x, y)].fg == t.sun))
                    .unwrap();
                let regen = (0..width).find(|&x| buf[(x, 14)].fg == t.rain).unwrap();
                assert_eq!(punkt, regen, "Stunde {stunde}, Breite {width}");
                for cell in &buf.content {
                    assert_eq!(cell.bg, t.bg);
                }
                terminal
                    .draw(|f| kurve(f, f.area(), &b.wetter, 24, Some(stunde), t))
                    .unwrap();
                let buf = terminal.backend().buffer();
                for y in 1..=14 {
                    assert_eq!(buf[(punkt, y)].fg, t.active);
                }
            }
        }
    }

    #[test]
    fn trockenhinweis_braucht_vollstaendige_nullwerte() {
        let mut b = crate::test_support::beispiel();
        b.wetter.hourly.precipitation_probability = vec![Some(0.0); 25];
        b.wetter.hourly.precipitation = vec![Some(0.0); 25];
        let mut terminal = Terminal::new(TestBackend::new(80, 16)).unwrap();
        terminal
            .draw(|f| kurve(f, f.area(), &b.wetter, 24, None, Theme::neu()))
            .unwrap();
        assert!(text(terminal.backend().buffer()).contains("kein Niederschlag prognostiziert"));
        b.wetter.hourly.precipitation_probability[4] = None;
        terminal
            .draw(|f| kurve(f, f.area(), &b.wetter, 24, None, Theme::neu()))
            .unwrap();
        let bild = text(terminal.backend().buffer());
        assert!(!bild.contains("kein Niederschlag"));
        assert!(bild.contains("fehlende Stundendaten"));
        b.wetter.hourly = Stunden::default();
        terminal
            .draw(|f| kurve(f, f.area(), &b.wetter, 24, None, Theme::neu()))
            .unwrap();
        assert!(text(terminal.backend().buffer()).contains("Keine Temperaturdaten"));
    }

    #[test]
    fn grosse_fenster_lassen_aussenraum_und_footer_unten() {
        for ansicht in 0..3 {
            let mut app = App::neu(Some(crate::test_support::beispiel()), true);
            app.ansicht = ansicht;
            let mut terminal = Terminal::new(TestBackend::new(220, 60)).unwrap();
            terminal.draw(|f| zeichnen(f, &app)).unwrap();
            let buf = terminal.backend().buffer();
            for y in 0..60 {
                for x in (0..44).chain(176..220) {
                    assert_eq!(buf[(x, y)].symbol(), " ");
                }
            }
            for y in 36..56 {
                for x in 0..220 {
                    assert_eq!(buf[(x, y)].symbol(), " ");
                }
            }
            assert!(text(buf).lines().nth(57).unwrap().contains("[q]"));
        }
    }

    #[test]
    fn details_bleiben_beim_scrollen_vollstaendig_und_sonne_braucht_daten() {
        let b = crate::test_support::beispiel();
        let mut terminal = Terminal::new(TestBackend::new(40, 16)).unwrap();
        let mut alle = String::new();
        for offset in [0, 8, 16, 24, 40] {
            terminal
                .draw(|f| details(f, f.area(), &b, offset, Theme::neu()))
                .unwrap();
            alle.push_str(&text(terminal.backend().buffer()));
        }
        for label in [
            "LUFT",
            "WIND",
            "SONNE",
            "WEITERE WERTE",
            "Taupunkt",
            "Bewölkung",
            "Böen",
            "Sonnenschein",
            "UV-Maximum",
            "Regen heute",
            "Luftdruck",
            "Sichtweite",
        ] {
            assert!(alle.contains(label), "Nicht erreichbar: {label}");
        }
        let mut w = b.wetter;
        w.daily.sunrise.clear();
        assert!(
            tageslicht(&w, 40, Theme::neu())
                .to_string()
                .contains("nicht verfügbar")
        );
    }

    #[test]
    fn kurve_laesst_luecken_und_verwendet_keine_tagesextreme() {
        let mut b = crate::test_support::beispiel();
        b.wetter.hourly.temperature_2m = vec![Some(-2.0), None, Some(4.0), Some(5.0)];
        let segments = temperatursegmente(&b.wetter, &[0, 1, 2, 3]);
        assert_eq!(
            segments,
            vec![vec![(0.0, -2.0)], vec![(2.0, 4.0), (3.0, 5.0)]]
        );
        b.wetter.hourly.temperature_2m.clear();
        assert!(temperatursegmente(&b.wetter, &[0, 1]).is_empty());
    }
    #[test]
    fn countdown_und_stunden_ueber_mitternacht() {
        let b = crate::test_support::beispiel();
        assert_eq!(sonnenstatus(&b.wetter), "Sonnenuntergang in 2h 13m.");
        assert_eq!(b.wetter.stunden_indices().len(), 25);
        let mut w = b.wetter.clone();
        w.daily.sunset.clear();
        assert!(sonnenstatus(&w).contains("nicht verfügbar"));
    }
    #[test]
    fn ansichten_bleiben_bei_groessenwechseln_und_fehlenden_daten_stabil() {
        for (width, height) in [
            (220, 60),
            (140, 42),
            (120, 36),
            (96, 38),
            (80, 30),
            (64, 24),
            (40, 18),
            (20, 8),
        ] {
            for view in 0..3 {
                let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
                let mut app = App::neu(Some(crate::test_support::beispiel()), true);
                app.ansicht = view;
                terminal.draw(|f| zeichnen(f, &app)).unwrap();
                let buffer = terminal.backend().buffer();
                let text = (0..height)
                    .map(|y| {
                        (0..width)
                            .map(|x| buffer[(x, y)].symbol())
                            .collect::<String>()
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                if width >= 40 {
                    assert!(text.contains("Schönengrund"));
                    assert!(text.contains("[q]"));
                }
                if view == 0 && width >= 80 && height >= 30 {
                    assert!(text.contains("TEMPERATUR"));
                    assert!(text.contains("AUSBLICK"));
                }
                if let Ok(dir) = std::env::var("WETTER_RENDER_DIR") {
                    std::fs::create_dir_all(&dir).unwrap();
                    std::fs::write(format!("{dir}/{width}x{height}-{view}.txt"), &text).unwrap();
                    let rgb = |c: Color| match c {
                        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
                        _ => "".to_owned(),
                    };
                    let cells = buffer
                        .content
                        .iter()
                        .map(|c| serde_json::json!([c.symbol(), rgb(c.fg), rgb(c.bg)]))
                        .collect::<Vec<_>>();
                    std::fs::write(
                        format!("{dir}/{width}x{height}-{view}.json"),
                        serde_json::to_vec(
                            &serde_json::json!({"width":width,"height":height,"cells":cells}),
                        )
                        .unwrap(),
                    )
                    .unwrap();
                }
                let b = app.bericht.as_mut().unwrap();
                b.wetter.current = Aktuell::default();
                b.wetter.hourly = Stunden::default();
                b.wetter.daily = Tage::default();
                terminal.draw(|f| zeichnen(f, &app)).unwrap();
            }
        }
    }
}
