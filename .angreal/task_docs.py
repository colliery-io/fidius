"""
Docs tasks: `angreal docs build`, `angreal docs serve` and `angreal docs api`.

The docs are an Astro Starlight site in docs/, with the Colliery theme
starlight-aurora. They need Node 22 or later, and plissken 0.2.1 for the API
reference: `plissken render` writes it into docs/src/content/docs/reference/api/
(see plissken.toml). Those pages are not in git; CI renders them the same way.
"""

import hashlib
import os
import shutil
import subprocess
import sys
from pathlib import Path

import angreal  # type: ignore

PROJECT_ROOT = Path(angreal.get_root()).parent
DOCS_ROOT = PROJECT_ROOT / "docs"

# The plissken version CI uses (.github/workflows/docs.yml).
PLISSKEN_VERSION = "0.2.1"
# plissken has no --version flag. The binaries of the v0.2.1 release have these
# sha256 sums; a `cargo install` build is checked with `cargo install --list`.
PLISSKEN_SHA256 = {
    "ee88d4725c1b93e564fbf568d513401184e400b1ed243bee4cee2f15c7b37426",  # aarch64-apple-darwin
    "698903ec9017eeffa0b608153089099df1e22188a8edd281cd3cde81b60e1a83",  # x86_64-apple-darwin
    "414beb801ba0856c6423b894135e56e360d1ba0a932169cd379a99a288a6ce46",  # x86_64-unknown-linux-gnu
    "8f9c2853dc147940fe383aeff0012534b8e7b592414eaa25f61da34d4ab0edd0",  # x86_64-unknown-linux-musl
    "b97c6f68e436250e1dbaec54135fea614d0152ed9a47fddc9b777d18b1aa9087",  # aarch64-unknown-linux-gnu
}
PLISSKEN_INSTALL = (
    f"install plissken {PLISSKEN_VERSION} from "
    f"https://github.com/colliery-io/plissken/releases/tag/v{PLISSKEN_VERSION} "
    f"or with `cargo install plissken --version {PLISSKEN_VERSION} --locked`"
)

docs = angreal.command_group(name="docs", about="documentation site commands")


def _is_cargo_install(plissken):
    cargo = shutil.which("cargo")
    if not cargo:
        return False
    cargo_home = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
    if Path(plissken).resolve() != (cargo_home / "bin" / "plissken").resolve():
        return False
    listed = subprocess.run([cargo, "install", "--list"], capture_output=True, text=True).stdout
    return f"plissken v{PLISSKEN_VERSION}:" in listed


def _plissken():
    plissken = shutil.which("plissken")
    if not plissken:
        print(f"plissken not found: {PLISSKEN_INSTALL}", file=sys.stderr)
        return 1
    digest = hashlib.sha256(Path(plissken).read_bytes()).hexdigest()
    if digest not in PLISSKEN_SHA256 and not _is_cargo_install(plissken):
        print(
            f"{plissken} is not plissken {PLISSKEN_VERSION}: {PLISSKEN_INSTALL}",
            file=sys.stderr,
        )
        return 1
    return subprocess.run([plissken, "render"], cwd=str(PROJECT_ROOT)).returncode


def _npm(*args):
    npm = shutil.which("npm")
    if not npm:
        print("npm not found: install Node 22 or later", file=sys.stderr)
        return 1
    if not (DOCS_ROOT / "node_modules").exists():
        result = subprocess.run([npm, "ci", "--no-audit", "--no-fund"], cwd=str(DOCS_ROOT))
        if result.returncode != 0:
            return result.returncode
    return subprocess.run([npm, *args], cwd=str(DOCS_ROOT)).returncode


@docs()
@angreal.command(
    name="build",
    about="render the API reference and build the site into docs/dist",
    tool=angreal.ToolDescription(
        "Runs `plissken render` (plissken 0.2.1) for the API reference, then "
        "builds the Starlight docs site in docs/ into docs/dist. Installs the "
        "npm dependencies on the first run. Fails when plissken is missing or "
        "not 0.2.1, or when a page does not build.",
        risk_level="safe",
    ),
)
def docs_build():
    return _plissken() or _npm("run", "build")


@docs()
@angreal.command(
    name="serve",
    about="render the API reference and serve the site with live reload",
    tool=angreal.ToolDescription(
        "Runs `plissken render` (plissken 0.2.1), then serves the docs site at "
        "http://localhost:4321 and reloads on each change. In a terminal it runs "
        "until Ctrl+C. From a script or an agent, Astro runs it in the "
        "background: stop it with `npx astro dev stop` in docs/.",
        risk_level="safe",
    ),
)
def docs_serve():
    return _plissken() or _npm("run", "dev")


@docs()
@angreal.command(
    name="api",
    about="render the API reference only (plissken render)",
    tool=angreal.ToolDescription(
        "Runs `plissken render` (plissken 0.2.1), which writes the API reference "
        "pages and _sidebar.mjs into docs/src/content/docs/reference/api/. "
        "Fails when plissken is missing or not 0.2.1.",
        risk_level="safe",
    ),
)
def docs_api():
    return _plissken()
