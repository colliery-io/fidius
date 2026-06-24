import angreal
import os
import shutil
import subprocess
import sys

# Coverage artifacts live here (git-ignored). HTML lands in <COV_DIR>/html/.
COV_DIR = "target/coverage"


@angreal.command(
    name="coverage",
    about="Measure test coverage with cargo-llvm-cov (per-crate summary + HTML + lcov)",
    long_about=(
        "Run the workspace test suite under cargo-llvm-cov source-based "
        "instrumentation and emit a per-crate summary table to stdout, an HTML "
        "report, and an lcov file under target/coverage/. Report-only: this never "
        "fails on a coverage threshold (FIDIUS-I-0033 Phase 1). The default feature "
        "surface is the workspace defaults plus `streaming` (`python` is left off, "
        "mirroring `angreal test`). The `wasm` feature is NOT instrumented by "
        "default: its tests build wasm32-wasip2 component fixtures at test time, and "
        "those sub-builds fail under cargo-llvm-cov's `-C instrument-coverage` flags "
        "(instrument-coverage is unsupported on the wasm target). The wasm path's "
        "correctness is covered by the non-instrumented `wasm` CI job / `angreal "
        "test`. Use --wasm to attempt it anyway (best-effort; expected to fail here)."
    ),
)
@angreal.argument(
    name="open",
    long="open",
    takes_value=False,
    help="Open the HTML report in a browser when done",
)
@angreal.argument(
    name="wasm",
    long="wasm",
    takes_value=False,
    help="Also instrument the `wasm` feature (best-effort; the at-test-time wasm32-wasip2 fixture builds fail under coverage instrumentation)",
)
def task_coverage(open=False, wasm=False):
    """Workspace coverage via cargo-llvm-cov: summary + HTML + lcov, report-only."""
    if shutil.which("cargo-llvm-cov") is None:
        print(
            "cargo-llvm-cov not found. Install it with:\n"
            "  cargo install cargo-llvm-cov\n"
            "  rustup component add llvm-tools-preview",
            file=sys.stderr,
        )
        return 1

    # Feature surface: workspace defaults + `streaming`. `wasm` is opt-in and
    # known to break (its tests spawn wasm32-wasip2 fixture builds that inherit
    # `-C instrument-coverage`, which the wasm target rejects). `python` stays off
    # (mirrors `angreal test`).
    feats = ["wasm", "streaming"] if wasm else ["streaming"]
    feat_args = ["--features", ",".join(feats)]

    os.makedirs(COV_DIR, exist_ok=True)

    # Run the suite once collecting coverage (--no-report), then render every
    # format from that single run so the HTML, lcov, and summary all agree.
    if subprocess.run(["cargo", "llvm-cov", "clean", "--workspace"]).returncode != 0:
        return 1
    run = subprocess.run(
        ["cargo", "llvm-cov", "--no-report", "--workspace"] + feat_args
    )
    if run.returncode != 0:
        print(
            "Tests failed under coverage instrumentation; report not generated.",
            file=sys.stderr,
        )
        return run.returncode

    subprocess.run(["cargo", "llvm-cov", "report", "--summary-only"])
    subprocess.run(
        ["cargo", "llvm-cov", "report", "--html", "--output-dir", COV_DIR]
    )
    subprocess.run(
        [
            "cargo",
            "llvm-cov",
            "report",
            "--lcov",
            "--output-path",
            os.path.join(COV_DIR, "lcov.info"),
        ]
    )

    html_index = os.path.join(COV_DIR, "html", "index.html")
    print(f"\nCoverage written:\n  HTML : {html_index}\n  lcov : {COV_DIR}/lcov.info")
    if open:
        opener = "open" if sys.platform == "darwin" else "xdg-open"
        subprocess.run([opener, html_index])
    return 0
