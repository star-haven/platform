default:
    @just --list

# Auto-format the source tree
fmt:
    treefmt

# Run the project locally
watch $RUST_BACKTRACE="1":
    cargo leptos watch --hot-reload

# Run cargo in release mode (prints red panic)
watch-release:
    cargo leptos watch --release

# Run tests (backend & frontend)
test:
    cargo watch -- cargo leptos test

# Migrate database and generate code for db entities
migrate:
    sea-orm-cli migrate
    sea-orm-cli generate entity -o entity/src --with-serde both --lib --date-time-crate time
