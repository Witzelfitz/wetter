#!/usr/bin/env python3
"""Build local release archives and a Homebrew formula; never publish anything."""
import argparse
import gzip
import hashlib
import io
import json
import re
import subprocess
import tarfile
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TARGETS = ("aarch64-apple-darwin", "x86_64-apple-darwin", "x86_64-unknown-linux-gnu")


def version():
    value = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["version"]
    if not re.fullmatch(r"\d+\.\d+\.\d+", value):
        raise ValueError("Only stable x.y.z versions are supported by this release workflow")
    return value


def filename(target):
    if target not in TARGETS:
        raise ValueError(f"Unsupported target: {target}")
    return f"wetter-v{version()}-{target}.tar.gz"


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def notices(target):
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", target], cwd=ROOT
    ))
    sections = ["Third-party licenses for wetter\nGenerated from Cargo.lock for " + target]
    for package in sorted(metadata["packages"], key=lambda p: (p["name"], p["version"])):
        if package["source"] is None:
            continue
        folder = Path(package["manifest_path"]).parent
        files = {p for glob in ("LICENSE*", "LICENCE*", "COPYING*", "NOTICE*", "license*", "licenses/*")
                 for p in folder.glob(glob) if p.is_file()}
        if package.get("license_file"):
            files.add(folder / package["license_file"])
        if package["name"] == "tui-big-text" and package["version"] == "0.8.10":
            files.add(ROOT / "packaging/licenses/tui-big-text-MIT.txt")
        if not files:
            raise ValueError(f"Missing license text for {package['name']} {package['version']}")
        sections.append(f"\n{'=' * 72}\n{package['name']} {package['version']}\n"
                        f"SPDX: {package.get('license', 'see files')}\n"
                        f"Source: {package.get('repository') or package['source']}")
        for path in sorted(files):
            sections.append(f"\n--- {path.name} ---\n{path.read_text(errors='replace')}")
    return "\n".join(sections).encode()


def package(binary, target, output):
    binary = binary.resolve()
    expected = f"wetter {version()}"
    actual = subprocess.check_output([str(binary), "--version"], text=True).strip()
    if actual != expected:
        raise ValueError(f"Binary version mismatch: expected {expected!r}, got {actual!r}")
    # The target must match the native binary. Cross-builds are packaged on their own runner.
    header = binary.read_bytes()[:64]
    architecture_ok = {
        "aarch64-apple-darwin": header[:8] == bytes.fromhex("cffaedfe0c000001"),
        "x86_64-apple-darwin": header[:8] == bytes.fromhex("cffaedfe07000001"),
        "x86_64-unknown-linux-gnu": header[:4] == b"\x7fELF" and header[4:6] == b"\x02\x01" and header[18:20] == b"\x3e\x00",
    }
    if not architecture_ok.get(target):
        raise ValueError(f"Binary architecture does not match {target}")
    output.mkdir(parents=True, exist_ok=True)
    archive = output / filename(target)
    files = [("wetter", binary.read_bytes(), 0o755)]
    for name in ("LICENSE", "README.md", "CHANGELOG.md", "THIRD_PARTY_NOTICES.md"):
        files.append((name, (ROOT / name).read_bytes(), 0o644))
    files.append(("THIRD_PARTY_LICENSES.txt", notices(target), 0o644))
    # Stable archive metadata: checksums do not change just because file mtimes did.
    with archive.open("wb") as raw:
        with gzip.GzipFile(fileobj=raw, mode="wb", filename="", mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode="w") as tar:
                for name, content, mode in files:
                    info = tarfile.TarInfo(name)
                    info.size = len(content)
                    info.mode = mode
                    info.mtime = 0
                    tar.addfile(info, io.BytesIO(content))
    archive.with_name(archive.name + ".sha256").write_text(f"{sha256(archive)}  {archive.name}\n")
    print(archive)


def formula(repository, artifacts, output):
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
        raise ValueError("Repository must be OWNER/REPO")
    replacements = {"VERSION": version(), "REPOSITORY": repository}
    checksums = []
    for target in TARGETS:
        path = artifacts / filename(target)
        digest = sha256(path)
        sidecar = path.with_name(path.name + ".sha256")
        if sidecar.read_text().strip() != f"{digest}  {path.name}":
            raise ValueError(f"Checksum mismatch: {path.name}")
        replacements[target] = digest
        checksums.append(f"{digest}  {path.name}\n")
    text = (ROOT / "packaging/homebrew/wetter.rb.in").read_text()
    for key, value in replacements.items():
        text = text.replace("@" + key + "@", value)
    if re.search(r"@[A-Za-z0-9_-]+@", text):
        raise ValueError("Unresolved Homebrew template placeholders")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(text)
    checksums.append(f"{sha256(output)}  {output.name}\n")
    (artifacts / "SHA256SUMS").write_text("".join(checksums))
    print(output)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    check = sub.add_parser("check-version")
    check.add_argument("tag")
    pack = sub.add_parser("package")
    pack.add_argument("--binary", type=Path, required=True)
    pack.add_argument("--target", choices=TARGETS, required=True)
    pack.add_argument("--output", type=Path, default=ROOT / "dist")
    brew = sub.add_parser("formula")
    brew.add_argument("--repository", required=True)
    brew.add_argument("--artifacts", type=Path, default=ROOT / "dist")
    brew.add_argument("--output", type=Path, default=ROOT / "dist/wetter.rb")
    args = parser.parse_args()
    if args.command == "check-version":
        if args.tag != f"v{version()}":
            parser.error(f"Tag must equal Cargo.toml version: v{version()}")
        print(version())
    elif args.command == "package":
        package(args.binary, args.target, args.output)
    else:
        formula(args.repository, args.artifacts, args.output)


if __name__ == "__main__":
    main()
