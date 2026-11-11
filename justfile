# Common development commands. Run `just` to list them.
#
# Recipes run with `sh` (on Windows, the one from Git for Windows).

# Homebrew's rustup doesn't put cargo on PATH, so add the active toolchain's bin directory
# (which also holds the cargo-fmt and cargo-clippy subcommands).
export PATH := if os_family() == "unix" { `dirname "$(command -v cargo || rustup which cargo)"` + ":" + env_var("PATH") } else { env_var("PATH") }

app := "target/release/bundle/osx/Streaker.app"
log := home_directory() / "Library/Application Support/com.jamieweavis.Streaker/streaker.log"

# List available commands
default:
    @just --list --unsorted

# Run a debug build (logs to the terminal)
run:
    cargo run

# Build a debug binary
build:
    cargo build

# Build an optimised binary
release:
    cargo build --release

# Run the tests
test:
    cargo test

# Format the code
fmt:
    cargo fmt

# Check formatting and run clippy
lint:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings

# Run everything CI runs
check: lint test

# Package the app (.app/.dmg on macOS, .deb on Linux, .msi on Windows)
bundle:
    @cargo bundle --version >/dev/null 2>&1 || cargo install cargo-bundle
    cargo bundle --release

# Install Streaker.app to /Applications and launch it
[macos]
install: bundle
    -pkill -f "Streaker.app/Contents/MacOS/Streaker"
    rm -rf /Applications/Streaker.app
    cp -R "{{ app }}" /Applications/
    open /Applications/Streaker.app

# Launch the bundled app without installing it
[macos]
open: bundle
    open "{{ app }}"

# Follow the app's log file
[macos]
logs:
    tail -f "{{ log }}"

# Remove build artifacts
clean:
    cargo clean
