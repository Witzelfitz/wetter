# Daten und Abhängigkeiten

Der wetter-Quellcode steht unter der [MIT-Lizenz](LICENSE). Diese Lizenz ersetzt nicht die Lizenzen der eingebundenen Abhängigkeiten oder der Wetterdaten.

## Rust-Abhängigkeiten

`Cargo.lock` dokumentiert die tatsächlich verwendeten Versionen. Release-Archive enthalten `THIRD_PARTY_LICENSES.txt`, das `scripts/release.py` aus den ausgelieferten Crate-Lizenzdateien zusammenstellt. Der Generator bricht ab, falls Lizenztexte fehlen. Für tui-big-text 0.8.10 ist dessen MIT-Lizenz separat unter `packaging/licenses/` hinterlegt, da diese Version im Crate-Archiv keinen eigenen Lizenztext mitliefert.

Insbesondere werden Ratatui, Crossterm, tui-big-text, Reqwest, Serde, Chrono und unicode-width verwendet. Die vollständigen Lizenztexte stehen im generierten Verzeichnistext, nicht nur in dieser Auswahl.

## Wetter und Orte

- Wetter: [Open-Meteo](https://open-meteo.com/), [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
- Orte: [GeoNames](https://www.geonames.org/) über die [Open-Meteo-Geocoding-API](https://open-meteo.com/en/docs/geocoding-api).
- Der verwendete kostenlose API-Dienst ist laut [Nutzungsbedingungen](https://open-meteo.com/en/terms) für nicht kommerzielle Nutzung vorgesehen. Der Code enthält keine Anbindung an den kommerziellen API-Endpunkt.

Anfragen werden direkt an Open-Meteo gesendet. Die App betreibt keinen eigenen Wetterserver und enthält keine Telemetrie.
