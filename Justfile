_default:
    cargo just --list -u

init:
    cargo tool --install

# ── Formatting ────────────────────────────────────────────────────────────────

format: format-fix

format-check:
    cargo fmt --all -- --check

format-fix:
    cargo fmt --all

# ── Linting ──────────────────────────────────────────────────────────────────

lint: lint-check

lint-check:
    cargo clippy --no-deps --all-targets --all-features -- -D warnings

lint-fix:
    cargo clippy --no-deps --all-targets --all-features --fix

# ── Build ────────────────────────────────────────────────────────────────────

compile:
    cargo check --all-features

doc:
    cargo doc --all-features --no-deps --open --lib

doc-gen:
    cargo clean --doc
    cargo doc --no-deps
    echo '<meta http-equiv="refresh" content="0;url=framepace/index.html">' > target/doc/index.html
    rm target/doc/.lock

# ── Test ─────────────────────────────────────────────────────────────────────

test *ARGS:
    cargo tool cargo-nextest run {{ARGS}}

test-doc *ARGS:
    cargo test {{ARGS}} --doc

test-all:
    cargo just test --all-features
    cargo just test-doc --all-features

# ── Coverage ─────────────────────────────────────────────────────────────────

coverage *ARGS:
    cargo tool cargo-llvm-cov --lib --open

coverage-gen:
    cargo tool cargo-llvm-cov --lib --lcov --output-path lcov.info

# ── Checks ───────────────────────────────────────────────────────────────────

semver-checks:
    cargo tool cargo-semver-checks

msrv:
    cargo tool cargo-hack check --rust-version --workspace --all-targets --ignore-private

# ── Composite ────────────────────────────────────────────────────────────────

fix:
    cargo just format-fix
    cargo just lint-fix

check:
    cargo just format
    cargo just lint
