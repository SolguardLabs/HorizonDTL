#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import json
import re
import struct
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXPECTED_BLOBS = {
    "src/ledger/state.rs": "4d7fe9283afc5cf3a3e2d03f2d0325d2662de9f1",
    "src/notes/order.rs": "130f5e38b66c593fd0d40b01f12ec502516f9e55",
    "src/amount/units.rs": "c41fb6ab2d24bb20ecb188cf3558d13861a20a09",
    "src/routes/book.rs": "8957ab93c1ca7a64a7e024570041cc580dbdecdf",
    "src/vault/state.rs": "9e5853308ec98de9c0053c6e4a3828083768dc39",
}
REQUIRED_DOCS = {
    "arquitectura.md",
    "integracion-sdk.md",
    "modelo-economico.md",
    "notas-y-tickets.md",
    "operacion.md",
    "riesgo-y-solvencia.md",
    "rutas-liquidacion.md",
}
PUBLIC_PATTERN = re.compile(
    r"\b(?:ctf|labs?|laboratorios?|vulnerabil(?:ity|idad|idades)|"
    r"exploits?|bugs?|attackers?|atacantes?)\b",
    re.IGNORECASE,
)
TEXT_SUFFIXES = {".rs", ".js", ".json", ".md", ".yml", ".yaml", ".toml", ".sh"}
TEXT_NAMES = {
    ".gitignore",
    ".prettierignore",
    ".prettierrc",
    ".editorconfig",
    ".gitattributes",
}


def fail(message: str, errors: list[str]) -> None:
    errors.append(message)


def git_blob(path: str) -> str:
    return subprocess.check_output(
        ["git", "hash-object", "--", path], cwd=ROOT, text=True
    ).strip()


def png_dimensions(path: Path) -> tuple[int, int]:
    data = path.read_bytes()[:24]
    if len(data) != 24 or data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("banner is not a PNG")
    return struct.unpack(">II", data[16:24])


def public_text_files() -> list[Path]:
    result: list[Path] = []
    for path in ROOT.rglob("*"):
        if not path.is_file():
            continue
        relative = path.relative_to(ROOT)
        if any(
            part in {".git", "node_modules", "target", "coverage"}
            for part in relative.parts
        ):
            continue
        if relative.as_posix() == "scripts/verify_release.py":
            continue
        if path.suffix in TEXT_SUFFIXES or path.name in TEXT_NAMES:
            result.append(path)
    return result


def main() -> int:
    errors: list[str] = []

    release = json.loads((ROOT / "RELEASE.json").read_text(encoding="utf-8"))
    expected_release = {
        "name": "Production 1.0.0",
        "version": "1.0.0",
        "tag": "v1.0.0",
        "branch": "production",
        "rust": "1.96",
        "bun": "1.3.14",
        "status": "stable",
    }
    if release != expected_release:
        fail("RELEASE.json does not match the stable contract", errors)

    cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    package = json.loads((ROOT / "package.json").read_text(encoding="utf-8"))
    if 'version = "1.0.0"' not in cargo or package.get("version") != "1.0.0":
        fail("manifest versions are not aligned at 1.0.0", errors)

    docs = {path.name for path in (ROOT / "docs").glob("*.md")}
    if docs != REQUIRED_DOCS:
        fail(f"documentation manifest differs: {sorted(docs)}", errors)
    diagram_count = sum(
        path.read_text(encoding="utf-8").count("```mermaid")
        for path in [
            ROOT / "README.md",
            ROOT / "SECURITY.md",
            *(ROOT / "docs").glob("*.md"),
        ]
    )
    if diagram_count < 20:
        fail(f"expected at least 20 Mermaid diagrams, found {diagram_count}", errors)

    banner = ROOT / "assets" / "banner.png"
    try:
        dimensions = png_dimensions(banner)
        if dimensions != (1672, 941):
            fail(f"unexpected banner dimensions: {dimensions}", errors)
    except (OSError, ValueError) as exc:
        fail(f"invalid banner: {exc}", errors)

    for path, expected in EXPECTED_BLOBS.items():
        observed = git_blob(path)
        if observed != expected:
            fail(f"protected domain changed: {path} ({observed})", errors)

    private_files = [
        path.relative_to(ROOT).as_posix()
        for path in ROOT.rglob("private_*")
        if ".git" not in path.parts and "target" not in path.parts
    ]
    if private_files:
        fail(f"private evidence found in public checkout: {private_files}", errors)

    for path in public_text_files():
        text = path.read_text(encoding="utf-8")
        match = PUBLIC_PATTERN.search(text)
        if match:
            fail(
                f"reserved public term in {path.relative_to(ROOT).as_posix()}: {match.group(0)}",
                errors,
            )

    required_paths = [
        ".github/dependabot.yml",
        ".github/workflows/ci.yml",
        ".github/workflows/release-integrity.yml",
        ".vscode/extensions.json",
        ".vscode/launch.json",
        ".vscode/settings.json",
        ".vscode/tasks.json",
        "client/horizon.js",
        "scripts/ci.sh",
        "scripts/tests.sh",
        "tests/solvency_model.rs",
    ]
    for relative in required_paths:
        if not (ROOT / relative).is_file():
            fail(f"required path is missing: {relative}", errors)

    ci = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
    for marker in [
        "ubuntu-latest",
        "windows-latest",
        "bash scripts/ci.sh",
        "production",
    ]:
        if marker not in ci:
            fail(f"CI marker is missing: {marker}", errors)

    docs_digest = hashlib.sha256()
    docs_to_hash = [
        ROOT / "README.md",
        ROOT / "SECURITY.md",
        *(ROOT / "docs").glob("*.md"),
    ]
    for path in sorted(docs_to_hash):
        docs_digest.update(path.relative_to(ROOT).as_posix().encode())
        docs_digest.update(path.read_bytes())

    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1

    print(
        "release contract valid: "
        f"docs={len(docs)} diagrams={diagram_count} "
        f"docs_sha256={docs_digest.hexdigest()}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
