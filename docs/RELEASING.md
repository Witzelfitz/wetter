# GitHub und Homebrew veröffentlichen

Dieser Ablauf ist für `Witzelfitz/wetter` und `Witzelfitz/homebrew-tap` vorbereitet. Das lokale Projekt allein erstellt oder veröffentlicht keine GitHub-Repositories. Ein Release-Workflow erzeugt zunächst einen **Entwurf**.

## 1. Repository erstmals veröffentlichen

Im Projektordner nach der lokalen Prüfung ausführen. Der Befehl `gh repo create --public` veröffentlicht den Code öffentlich.

```bash
cargo fmt --all --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
python3 -m unittest discover -s scripts -p 'test_*.py' -v

git status --short
# .idea, target, dist und .env-Dateien werden ignoriert.
git add .
git diff --cached --stat
git commit -m "Prepare wetter 0.1.0 for GitHub and Homebrew"
git branch -M main
gh repo create Witzelfitz/wetter --public --source . --remote origin --push
```

Existiert das Repository bereits, `gh repo create` überspringen und den passenden `origin` verwenden. Eine GitHub-Anmeldung über `gh auth login` ist für diese Veröffentlichung erforderlich.

Die CI startet nach dem Push. In GitHub unter **Actions → CI** prüfen, ob alle Jobs erfolgreich sind. Die CI arbeitet ohne API-Schlüssel und ohne Wetter-API-Aufrufe.

## 2. Ersten Release bauen lassen

Version in `Cargo.toml` und Eintrag in `CHANGELOG.md` müssen passen. `Cargo.lock` gehört ins Repository. Für die vorbereitete erste Version:

```bash
git tag -a v0.1.0 -m "wetter 0.1.0"
git push origin v0.1.0
```

**Actions → Release** führt Tests aus und baut:

- `wetter-v0.1.0-aarch64-apple-darwin.tar.gz`
- `wetter-v0.1.0-x86_64-apple-darwin.tar.gz`
- `wetter-v0.1.0-x86_64-unknown-linux-gnu.tar.gz`

Anschliessend werden `SHA256SUMS` und `wetter.rb` aus diesen Archiven erzeugt. Der Workflow lehnt abweichende Versions-Tags, manipulierte Prüfsummen und das Ersetzen eines bereits veröffentlichten Releases ab.

Der Workflow benötigt nur den automatisch bereitgestellten `GITHUB_TOKEN`. Ausschliesslich der Job zum Anlegen des Release-Entwurfs erhält `contents: write`. Ein persönlicher Token ist nicht erforderlich. Falls Actions im Repository deaktiviert ist, zunächst aktivieren.

## 3. Entwurf prüfen und veröffentlichen

Unter **Releases** den Entwurf öffnen, Release-Notizen ergänzen und die drei Archive, `SHA256SUMS` sowie `wetter.rb` prüfen. Mindestens das Archiv des eigenen Systems herunterladen, entpacken und starten:

```bash
./wetter --version
./wetter --help
./wetter --plain Zürich
./wetter --tui Zürich
```

Erst danach **Publish release** wählen. Die Homebrew-Downloadadressen funktionieren erst nach der Veröffentlichung. Die Homebrew-Formel selbst ist kein Platzhalter: Sie enthält die tatsächlichen Prüfsummen der gerade gebauten Dateien.

Die macOS-Binaries werden mit Deployment-Ziel 14.0 gebaut. Linux wird auf Ubuntu 22.04 gebaut. CI muss die anderen Systeme bestätigen; ein erfolgreicher lokaler Mac-Build allein beweist deren Funktion nicht. Apple-Notarisierung ist nicht Teil dieses Workflows.

## 4. Eigenen Homebrew-Tap einrichten

Im übergeordneten Ordner ein separates Repository anlegen:

```bash
gh repo create Witzelfitz/homebrew-tap --public --clone
cd homebrew-tap
mkdir -p Formula
gh release download v0.1.0 --repo Witzelfitz/wetter --pattern wetter.rb --dir Formula
```

Die vorbereitete Datei `packaging/homebrew/README.md` aus dem Wetter-Projekt als `README.md` in den Tap kopieren. Die MIT-Datei `LICENSE` ebenfalls übernehmen. `Formula/wetter.rb` kurz prüfen und dann versionieren:

```bash
git add README.md LICENSE Formula/wetter.rb
git commit -m "Add wetter 0.1.0"
git branch -M main
git push -u origin main
brew install Witzelfitz/tap/wetter
brew test Witzelfitz/tap/wetter
wetter --version
```

Wenn `wetter` bereits per Cargo installiert ist, können zwei ausführbare Dateien im PATH liegen. `command -v wetter` zeigt, welche verwendet wird. Für einen eindeutigen Homebrew-Test kann `"$(brew --prefix Witzelfitz/tap/wetter)/bin/wetter" --version` verwendet werden.

Die Datei `packaging/homebrew/wetter.rb.in` ist nur die Generatorvorlage. Sie darf nicht als fertige Formel in den Tap kopiert werden.

## 5. Spätere Versionen

1. `Cargo.toml` und `CHANGELOG.md` aktualisieren, mit `cargo check` die Paketversion im Lockfile nachziehen.
2. Tests ausführen und die Änderungen committen/pushen.
3. Einen neuen, passenden Tag setzen und pushen (zum Beispiel `v0.1.1`). Bestehende Release-Tags nicht verschieben.
4. Den neuen Release-Entwurf prüfen und veröffentlichen.
5. Die neue `wetter.rb` aus dem Release in `Formula/` des Taps übernehmen und dort committen/pushen.

Nutzer erhalten diese Version danach mit `brew update` und `brew upgrade Witzelfitz/tap/wetter`. Das Aktualisieren des Taps ist bewusst ein eigener Schritt; es wird kein organisationsübergreifender Schreib-Token benötigt.

## Lokal ein Release-Archiv prüfen

Beispiel für Apple Silicon:

```bash
MACOSX_DEPLOYMENT_TARGET=14.0 cargo build --locked --release --target aarch64-apple-darwin
python3 scripts/release.py check-version v0.1.0
python3 scripts/release.py package \
  --binary target/aarch64-apple-darwin/release/wetter \
  --target aarch64-apple-darwin
```

Das Archiv und seine Prüfsumme liegen in `dist/`. Es enthält das Programm, README, Changelog, MIT-Lizenz und gesammelte Lizenztexte der Rust-Abhängigkeiten. Der Paketierer prüft die Versionsausgabe und die tatsächliche Binärarchitektur.

Wenn die Archive aller drei Systeme in `dist/` vorliegen:

```bash
python3 scripts/release.py formula --repository Witzelfitz/wetter
ruby -c dist/wetter.rb
```

Der Generator erfindet keine Prüfsummen: Fehlt ein Archiv oder passt seine Prüfsumme nicht, bricht er ab.
