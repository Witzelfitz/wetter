# wetter

**Dein Wetter im Terminal.** Eine deutschsprachige CLI mit grosser Temperaturanzeige, ASCII-Wettersymbolen und einer interaktiven Oberfläche aus Ratatui.

```text
wetter Zürich                 Einmaliger Wetterbericht
wetter --tui Schönengrund      Interaktive Wetter-App
wetter --plain Berlin         ASCII-Text ohne Farben
wetter --json "New York"       JSON für Skripte
```

## Was du bekommst

- Aktuelles Wetter, gefühlte Temperatur, Wind und Böen.
- Temperaturkurven aus echten Stundendaten: zwölf Stunden in der Übersicht, 24 Stunden mit Stundenauswahl.
- Separate Niederschlagsleiste sowie Menge und Wahrscheinlichkeit.
- Drei-Tage-Vorschau, Sonnenuntergang und zusätzliche Wetterdetails.
- Übersicht, Stunden und Details; anpassbare Layouts für unterschiedliche Terminalgrössen.
- Ortsauswahl bei mehreren Treffern und Aktualisierung im Hintergrund.
- Vorhandene Daten bleiben bei fehlgeschlagenen Aktualisierungen sichtbar. Fehlende Werte werden als `—` beziehungsweise `–` dargestellt.

## Installation

### Lokal aus dem Quellcode

Rust ab **1.88** und eine Internetverbindung zum Laden der Abhängigkeiten sind erforderlich.

```bash
cargo install --path . --locked
wetter --version
wetter --tui
```

Cargo installiert den Befehl normalerweise nach `~/.cargo/bin`. Dieser Ordner muss im `PATH` liegen. Für eine neue lokale Version den Installationsbefehl erneut ausführen.

Zum Entwickeln reicht:

```bash
cargo run --quiet -- --tui Zürich
```

### Homebrew

Installation über den eigenen [Homebrew-Tap](https://github.com/Witzelfitz/homebrew-tap):

```bash
brew install Witzelfitz/tap/wetter
wetter --tui Zürich
```

Updates:

```bash
brew update
brew upgrade Witzelfitz/tap/wetter
```

Die Formel verwendet fertige Programmdateien; eine Rust-Installation ist dafür nicht nötig. Vorbereitet sind macOS 14+ auf Apple Silicon und Intel sowie Linux x86-64 (Ubuntu 22.04 oder neuer beziehungsweise kompatibles glibc-System). Linux ARM und Windows sind noch keine Release-Ziele.

### Direkter Download

Fertige Archive stehen unter [GitHub Releases](https://github.com/Witzelfitz/wetter/releases) bereit:

| System | Archiv-Endung |
| --- | --- |
| macOS, Apple Silicon | `aarch64-apple-darwin.tar.gz` |
| macOS, Intel | `x86_64-apple-darwin.tar.gz` |
| Linux, x86-64 | `x86_64-unknown-linux-gnu.tar.gz` |

Archiv und `SHA256SUMS` derselben Version herunterladen. Den passenden Eintrag aus `SHA256SUMS` mit `shasum -a 256 ARCHIV` vergleichen (Linux: `sha256sum ARCHIV`). Danach das Archiv entpacken und `./wetter --version` ausführen. Die Datei `wetter` kann in einen eigenen Ordner im `PATH` gelegt werden, etwa `~/.local/bin`.

## Bedienung

Ohne `--tui` zeigt das Programm einen Bericht und beendet sich. Ohne Ortsangabe fragt es im Terminal nach einem Ort. `--json` und Aufrufe ohne interaktive Eingabe benötigen eine Ortsangabe.

| Taste in der TUI | Funktion |
| --- | --- |
| `Tab`, `Shift+Tab`, `1`, `2`, `3` | Ansicht wechseln |
| `←` / `→` | Ansicht wechseln; in „Stunden“ eine Stunde wählen |
| `↑` / `↓` | Ortsliste bedienen oder Details scrollen |
| `/` | Neuen Ort suchen |
| `r` | Aktuellen Ort aktualisieren |
| `?` | Hilfe öffnen |
| `q`, `Esc`, `Ctrl+C` | Beenden; `Esc` schliesst zuerst einen Dialog |

In RustRover die **Terminal-Ansicht** verwenden. Die normale Run-Konsole unterstützt Vollbild-Terminalsteuerung möglicherweise nicht vollständig. Ein Terminal mit Unicode-Schrift ist für die TUI sinnvoll; `--plain` verwendet ausschliesslich ASCII. `NO_COLOR=1` deaktiviert Farben.

Bei mehreren Treffern kannst du im Terminal auswählen. JSON und umgeleitete Ausgabe verwenden den bevölkerungsreichsten Treffer. Mit `--pick N` lässt sich für die einmalige Ausgabe ein anderer Treffer auswählen; die tatsächlich gewählte Stadt steht im Ergebnis.

```bash
wetter --pick 2 Zürich
wetter --help
wetter --version
```

## JSON und Skripte

```bash
wetter --json Zürich > wetter.json
wetter --json Zürich | jq '.weather.current.temperature_2m'
```

Die Ausgabe enthält `schema_version`, `location`, `weather`, `units` und `source`. Zeitangaben beziehen sich auf die Zeitzone in `weather.timezone`; Wetterwerte können `null` sein. `schema_version: 1` kennzeichnet das aktuelle Format.

Erfolg liefert Exit-Code `0`, Fehler liefern `1`. Fehlermeldungen stehen auf stderr. Hilfe, Version und die automatisierten CLI-Tests benötigen keine Wetter-API-Verbindung.

## Wetterdaten und Nutzung

Wetterdaten: [Open-Meteo](https://open-meteo.com/), Ortsdaten: [GeoNames](https://www.geonames.org/) über die Open-Meteo-Geocoding-API. Es handelt sich um Modelldaten, nicht um garantierte Messwerte an der Haustür. Die Ortszeit und der Modellzeitpunkt stehen im Bericht.

Der Code steht unter MIT. Davon getrennt gelten die [Open-Meteo-Nutzungsbedingungen](https://open-meteo.com/en/terms): Der hier verwendete kostenlose API-Endpunkt ist für nicht kommerzielle Nutzung vorgesehen. Die Wetterdaten werden unter CC BY 4.0 bereitgestellt. Das Tool sendet Ortsanfragen und Koordinaten direkt an Open-Meteo und benötigt keine eigenen Zugangsdaten. Es enthält keine Telemetrie und speichert bislang keine Einstellungen oder Wetterdaten dauerhaft.

## Entwicklung und Releases

```bash
cargo fmt --all --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
python3 -m unittest discover -s scripts -p 'test_*.py' -v
```

Die Python-Release-Werkzeuge benötigen Python 3.11 oder neuer und verwenden nur die Standardbibliothek. CI ist für macOS ARM/Intel, Linux x86-64 und Rust 1.88 vorbereitet.

- [Erstes GitHub-Repository, Release und Homebrew-Tap veröffentlichen](docs/RELEASING.md)
- [Beitragen und Projektstruktur](CONTRIBUTING.md)
- [Änderungen](CHANGELOG.md)
- [MIT-Lizenz](LICENSE) und [Drittlizenzen](THIRD_PARTY_NOTICES.md)
