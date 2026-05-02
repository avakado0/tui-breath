# Quick Start Guide

## Installation & Build

### 1. Install Rust (if needed)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Navigate to project
```bash
cd /home/draco/docs/tui_breath
```

### 3. Build
```bash
cargo build --release    # Optimized binary (~8MB)
# or
cargo build              # Debug build (faster compile)
```

### 4. Run
```bash
cargo run                             # Debug from source
./target/release/tui_breath          # Release binary
./target/debug/tui_breath            # Debug binary
```

## First Session

1. Select a pattern with `j/k` or arrow keys
2. Press `Enter` to continue
3. Adjust duration (minutes) and tempo (0.5x-2.0x) with `+/-`
4. Press `Tab` to switch fields
5. Press `Enter` to start
6. Press `p` to pause, `e` to end early
7. Press `s` to save session

## Testing

```bash
cargo test                    # Run all tests
cargo test --lib            # Library tests only
cargo test breathing::tests  # Specific test module
```

## Troubleshooting

**Cargo not found:**
```bash
which cargo
rustc --version
```

**Compilation errors:**
```bash
cd /home/draco/docs/tui_breath
cargo clean
cargo build
```

**Terminal too small:** Maximize terminal to at least 60×24 characters

**Sessions not saving:**
```bash
# Linux/macOS
mkdir -p ~/.local/share/tui_breath/sessions
chmod 755 ~/.local/share/tui_breath

# Windows
mkdir %APPDATA%\Local\tui_breath\sessions
```

## Debug Mode

```bash
RUST_BACKTRACE=1 cargo run    # Run with backtraces
```

## What Each Command Does

| Command | Purpose |
|---------|---------|
| `cargo build` | Debug build (faster compile) |
| `cargo build --release` | Optimized binary |
| `cargo run` | Build and run debug binary |
| `cargo test` | Run unit tests |
| `cargo clean` | Remove build artifacts |
| `cargo fix` | Auto-fix warnings |

## Files & Locations

**Configuration:**
- `Cargo.toml` - Dependencies and build settings

**Source Code:**
- `src/main.rs` - Entry point
- `src/app.rs` - State machine
- `src/engine/` - Breathing logic
- `src/ui/` - User interface screens
- `src/storage/` - Data persistence

**Session Data:**
- Linux/macOS: `~/.local/share/tui_breath/sessions/`
- Windows: `%APPDATA%\Local\tui_breath\sessions\`

## System Requirements

- **Rust 1.70+**
- **Terminal**: Any ANSI-compatible terminal (60×24+ size)
- **OS**: Linux, macOS, Windows 10+, or SSH session
- **Disk**: ~500KB for app + sessions

## Next Steps

1. Read [README.md](../README.md) for features overview
2. Read [AUDIO.md](AUDIO.md) for beeping feature guide
3. Read [IMPLEMENTATION_NOTES.md](IMPLEMENTATION_NOTES.md) for architecture
