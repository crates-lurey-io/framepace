default:
    @just --list

# ── Setup ────────────────────────────────────────────────────────────────────

setup-tools:
    cargo bin --install

# ── Formatting ────────────────────────────────────────────────────────────────

format:
    cargo fmt --all -- --check

fmt-check: format

fmt-fix:
    cargo fmt --all

# ── Linting ──────────────────────────────────────────────────────────────────

lint:
    cargo clippy --no-deps --all-targets --all-features -- -D warnings

lint-fix:
    cargo clippy --no-deps --all-targets --all-features --fix

# ── Build ────────────────────────────────────────────────────────────────────

compile:
    cargo check --all-features

doc:
    cargo doc --no-deps --document-private-items --all-features

doc-gen:
    cargo clean --doc
    cargo doc --no-deps
    echo '<meta http-equiv="refresh" content="0;url=framepace/index.html">' > target/doc/index.html
    rm target/doc/.lock

# ── Test ─────────────────────────────────────────────────────────────────────

test *args:
    cargo nextest run {{args}}

test-doc *args:
    cargo test {{args}} --doc

test-all:
    just test --all-features
    just test-doc --all-features

# ── Coverage ─────────────────────────────────────────────────────────────────

coverage *args:
    cargo llvm-cov --lib --open {{args}}

coverage-gen:
    cargo llvm-cov --lib --lcov --output-path lcov.info

msrv:
    cargo hack check --rust-version --all-targets

# ── Composite ────────────────────────────────────────────────────────────────

fix:
    just fmt-fix
    just lint-fix

check:
    just format
    just lint
