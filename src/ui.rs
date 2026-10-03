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
    let area = f.area().inner(Margin::new(2, 1));
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
            Constraint::Min(12),
            Constraint::Length(5),
            Constraint::Length(6),
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
            Constraint::Min(5),
            Constraint::Length(if area.width < 54 { 7 } else { 4 }),
            Constraint::Length(6),
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
            Constraint::Min(8),
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Length(if area.width < 65 { 7 } else { 16 }),
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

fn kurve(f: &mut Frame, area: Rect, w: &Wetter, horizont: usize, auswahl: Option<usize>, t: Theme) {
    if area.height < 5 {
        return;
    }
    let indices: Vec<_> = w.stunden_indices().into_iter().take(horizont + 1).collect();
    let segments = temperatursegmente(w, &indices);
    let bounds: Vec<f64> = segments.iter().flatten().map(|(_, y)| *y).collect();
    if bounds.is_empty() {
        f.render_widget(
            Paragraph::new("STUNDENVERLAUF\nKeine Temperaturdaten verfügbar.")
                .style(Style::default().fg(t.muted)),
            area,
        );
        return;
    }
    let low = bounds.iter().copied().fold(f64::INFINITY, f64::min).floor() - 1.0;
    let high = bounds
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max)
        .ceil()
        + 1.0;
    let index = auswahl.unwrap_or(0).min(indices.len().saturating_sub(1));
    let markierung = vec![(index as f64, low), (index as f64, high)];
    let mut sets: Vec<_> = segments
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
        .collect();
    if auswahl.is_some() {
        sets.push(
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(t.active))
                .data(&markierung),
        );
    }
    let start = indices
        .first()
        .and_then(|&i| w.hourly.time.get(i))
        .map_or("–", |s| uhrzeit(Some(s)));
    let chart_area = Rect {
        height: area.height.saturating_sub(2),
        ..area
    };
    let chart = Chart::new(sets)
        .block(
            Block::default()
                .title(format!("TEMPERATUR · {horizont} H AB {start} (ORTSZEIT)"))
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(t.border))
                .title_style(Style::default().fg(t.muted)),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, horizont as f64])
                .labels([
                    start.to_owned(),
                    format!("+{} h", horizont / 2),
                    format!("+{horizont} h"),
                ])
                .style(Style::default().fg(t.muted)),
        )
        .y_axis(
            Axis::default()
                .bounds([low, high])
                .labels([
                    format!("{low:.0}°"),
                    format!("{:.0}°", (low + high) / 2.0),
                    format!("{high:.0}°"),
                ])
                .style(Style::default().fg(t.muted)),
        );
    f.render_widget(chart, chart_area);
    // Separate Leiste: Wahrscheinlichkeiten, keine erfundenen Regenmengen.
    let compact = area.width < 66;
    let mut spans = vec![Span::styled(
        if compact { "Nied. % " } else { "Niederschlag " },
        Style::default().fg(t.muted),
    )];
    for (n, &i) in indices.iter().enumerate() {
        let chance = w.hourly.precipitation_probability.get(i).copied().flatten();
        let symbol = match chance {
            Some(v) if v.is_finite() => ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"]
                [((v.clamp(0.0, 100.0) / 100.0) * 7.0).round() as usize],
            _ => "·",
        };
        spans.push(Span::styled(
            if compact {
                symbol.to_owned()
            } else {
                format!("{symbol} ")
            },
            Style::default().fg(if auswahl == Some(n) { t.active } else { t.rain }),
        ));
    }
    f.render_widget(
        Paragraph::new(vec![
            Line::from(spans),
            Line::from(Span::styled(
                "Risiko 0–100 % · fehlende Werte: ·",
                Style::default().fg(t.muted),
            )),
        ]),
        Rect {
            x: area.x,
            y: area.bottom() - 2,
            width: area.width,
            height: 2,
        },
    );
}

fn stunden(f: &mut Frame, area: Rect, b: &Bericht, app: &App, t: Theme) {
    let parts = Layout::vertical([Constraint::Min(6), Constraint::Length(5)]).split(area);
    kurve(f, parts[0], &b.wetter, 24, Some(app.stunde), t);
    let w = &b.wetter;
    if let Some(&i) = w.stunden_indices().get(app.stunde) {
        let zeit = w
            .hourly
            .time
            .get(i)
            .map_or("–", String::as_str)
            .replace('T', " ");
        let text = format!(
            "{} · {}\nTemperatur {}   Wind {}\nNiederschlag: {} Risiko · {} in der vorangehenden Stunde\n← → Stunde wählen · Tab Ansicht wechseln",
            zeit,
            wettertext(w.hourly.weather_code.get(i).copied().flatten()),
            wert(w.hourly.temperature_2m.get(i).copied().flatten(), " °C"),
            wert(w.hourly.wind_speed_10m.get(i).copied().flatten(), " km/h"),
            ganz(
                w.hourly.precipitation_probability.get(i).copied().flatten(),
                " %"
            ),
            wert(w.hourly.precipitation.get(i).copied().flatten(), " mm")
        );
        f.render_widget(
            Paragraph::new(text)
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(t.active)),
            parts[1],
        );
    }
}

fn details(f: &mut Frame, area: Rect, b: &Bericht, offset: u16, t: Theme) {
    let w = &b.wetter;
    let a = &w.current;
    let text = format!(
        "JETZT · {}\n\nLuftfeuchte        {}\nTaupunkt           {}\nLuftdruck          {} (Meereshöhe)\nSichtweite         {}\nBewölkung          {}\nWindböen           {}\n\nHEUTE · TAGESPROGNOSE\n\nSonnenaufgang      {}\nSonnenuntergang    {}\nTageslicht         {}\nSonnenschein       {}\nUV-Maximum         {}\nNiederschlag       {}\n\nAlle Zeiten: {}\n— = keine Daten · ↑ ↓ zum Scrollen",
        a.time.replace('T', " "),
        ganz(a.relative_humidity_2m, " %"),
        wert(a.dew_point_2m, " °C"),
        ganz(a.pressure_msl, " hPa"),
        wert(a.visibility.map(|m| m / 1000.0), " km"),
        ganz(a.cloud_cover, " %"),
        wert(a.wind_gusts_10m, " km/h"),
        uhrzeit(w.tageszeit(&w.daily.sunrise)),
        uhrzeit(w.tageszeit(&w.daily.sunset)),
        dauer(w.tageswert(&w.daily.daylight_duration)),
        dauer(w.tageswert(&w.daily.sunshine_duration)),
        wert(w.tageswert(&w.daily.uv_index_max), ""),
        wert(w.tageswert(&w.daily.precipitation_sum), " mm"),
        w.timezone
    );
    let max_scroll = (text.lines().count() as u16).saturating_sub(area.height);
    f.render_widget(
        Paragraph::new(text)
            .scroll((offset.min(max_scroll), 0))
            .style(t.basis()),
        area,
    );
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
        Dialog::Hilfe=>f.render_widget(Paragraph::new("Tab / 1 2 3   Übersicht, Stunden, Details\n← →           Ansicht; in Stunden: Stunde wählen\n↑ ↓           Details scrollen / Ort wählen\n/             Neuen Ort suchen\nr             Wetter aktualisieren\nq / Esc       Beenden (im Dialog: zurück)\nCtrl+C        Sofort beenden\n\nGelb: Temperatur · Blau: Niederschlagsrisiko\nTürkis: aktive Auswahl · —: fehlende Daten\nZeiten am gewählten Ort, Stand laut Wettermodell.\n\nBeliebige Taste: zurück").style(t.basis().bg(t.surface)).wrap(Wrap{trim:true}),inner),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

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
        for (width, height) in [(120, 36), (96, 38), (80, 30), (64, 24), (40, 18), (20, 8)] {
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
