# Beitragen

Rust stable (mindestens 1.88) und Python 3.11+ für Release-Tests installieren. `rust-toolchain.toml` ergänzt rustfmt und Clippy.

```bash
cargo run -- --tui Zürich
cargo fmt --all
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
python3 -m unittest discover -s scripts -p 'test_*.py' -v
```

## Struktur

- `src/main.rs`: CLI-Argumente, einmalige Ausgabe, ASCII und JSON.
- `src/api.rs`, `src/data.rs`: HTTP-Abfragen und Datenmodelle.
- `src/app.rs`: Zustände, Eingaben und Hintergrundabfragen.
- `src/ui.rs`, `src/theme.rs`, `src/format.rs`: Layout, Farben und Formatierung.
- `src/test_support.rs`, `tests/`: deterministische Beispieldaten und CLI-Tests.
- `scripts/release.py`: Archivierung und Homebrew-Generator.
- `.github/workflows/`: CI und Release-Entwürfe.

Die Tests sollen ohne Wetter-API und ohne persönliche Zugangsdaten laufen. Für neue Datenfelder deren Einheit und Zeitbezug prüfen. Fehlende Werte dürfen nicht als Nullwerte erscheinen; bestehende Daten sollen bei einem Aktualisierungsfehler sichtbar bleiben.

Bei UI-Änderungen mehrere Grössen prüfen: 220×60, 140×42, 120×36, 80×30 und 64×24. Mit `WETTER_RENDER_DIR=/tmp/wetter-render cargo test` lassen sich die Testansichten als Textdateien und JSON-Zellraster inklusive Farben ausgeben. Für farbige Vorschauen darf `NO_COLOR` nicht gesetzt sein.

Pull Requests sollten Problem, Änderung und relevante Prüfung kurz beschreiben. Cargo.lock mit einchecken. IDE-Dateien, Zugangsdaten, `target/` und `dist/` bleiben lokal.
