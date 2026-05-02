# Audio Beeping Feature

## Quick Overview

Press `b` at any time to toggle audio feedback. You'll see:
- 🔊 = Beeping ON (will beep at phase changes)
- 🔇 = Beeping OFF (silent mode)

Beeps play automatically when transitioning between phases during a breathing session.

## Usage

### Enable/Disable Beeping
- Press `[b]` on any screen
- Status appears in the footer: `[b] 🔊` or `[b] 🔇`
- Setting persists across sessions

### When Beeps Occur
During an active breathing session:
- One short beep per phase transition
- Example: Inhale → Hold (beep), Hold → Exhale (beep), etc.
- Non-blocking: beeps play in background threads

## User Workflow Example

**5-minute Box Breathing with Beeping**

1. Start app: `cargo run`
2. Press `[b]` to enable beeping (see 🔊)
3. Select: **Box Breathing**
4. Set duration: **5 minutes**
5. Set tempo: **1.0x** (normal)
6. Press `Enter` to start
7. Listen for beeps at phase transitions:
   ```
   Phase 1: INHALE (0-4s)
   Phase 2: HOLD (4-8s)        🔊 BEEP!
   Phase 3: EXHALE (8-12s)      🔊 BEEP!
   Phase 4: HOLD (12-16s)       🔊 BEEP!
   [Cycle repeats]
   ```
8. Press `[b]` during session to toggle beeping on/off
9. Session completes, press `[s]` to save

## Technical Details

### How It Works

**Phase Change Detection:**
```
Session tick → Check if current_phase_idx changed
            ↓
            If yes AND beeping enabled → Spawn beep thread
            ↓
            Thread prints terminal bell (\x07)
            ↓
            Terminal plays audible beep
```

**Thread Safety:**
- Uses `Arc<Mutex<bool>>` for safe state sharing
- Beeps spawn in background (non-blocking UI)
- No async locks, simple and fast

### Sound Behavior

| System | Sound |
|--------|-------|
| Linux | Audible beep via speaker |
| macOS | System beep sound |
| Windows | Terminal beep |
| SSH | Beep transmitted over connection |
| No audio system | Visual bell (screen flash) |

### Why This Approach?

The audio beeping uses the terminal bell character because it:
- ✅ Works on ALL platforms (no external audio libraries)
- ✅ Works over SSH and remote connections
- ✅ Lightweight (single character)
- ✅ Simple and reliable
- ✅ Respects system audio settings
- ✅ No performance impact

## Customization

### Change Beep Behavior

Edit `src/audio.rs` in the `play_beep()` function:

**Multiple beeps:**
```rust
// Play 2 quick beeps
print!("\x07\x07");
```

**Different timing:**
```rust
print!("\x07");
std::thread::sleep(Duration::from_millis(50));
```

**Specific phases only:**
Edit `src/app.rs` in `on_tick()` to check phase type before beeping:
```rust
// Only beep on exhale start
if session_state.manager.engine.current_phase_idx == exhale_idx {
    self.beeper.beep();
}
```

Then rebuild:
```bash
cargo build --release
```

## Tips & Tricks

### 1. Stay Focused Without Watching
- Enable beeping before starting
- Close your eyes
- Follow auditory cues for phase changes
- Keep hands free

### 2. Silent Mode for Quiet Environments
- Press `[b]` to disable (🔇)
- Visual feedback continues
- No audio distraction

### 3. Test Beeping First
1. Toggle beeping ON (🔊)
2. Start a short 1-minute session
3. Listen for beeps at transitions
4. Adjust system volume if needed

### 4. System Volume Control
- Beeping uses terminal bell (system-level)
- Control with your OS volume settings
- Some terminals: Settings → Sound → Bell

## Troubleshooting

**No sound during session?**
1. Check beeping is ON (🔊 visible)
2. Check system volume is not muted
3. Check terminal supports beeps (try `echo -e '\x07'`)
4. Try different terminal if available

**Sound too quiet/loud?**
- Adjust OS volume settings
- Terminal settings may have separate bell volume

**Want different sound?**
- See "Customization" section above
- Edit `src/audio.rs` and rebuild

## Files & Implementation

### Modified Files
- **src/audio.rs** (NEW) - Beeper struct, 50 lines
- **src/main.rs** - Added `mod audio;`
- **src/app.rs** - Added beeper field, phase detection
- **src/ui/menu.rs** - Status display
- **src/ui/session.rs** - Status display with toggle hint

### Testing

**Verify compilation:**
```bash
cargo test
# Expected: 4 passed, 0 failed
```

**Manual testing:**
1. Build: `cargo build --release`
2. Run: `./target/release/tui_breath`
3. Start a session
4. Listen for beeps at phase transitions
5. Press `b` to toggle on/off

## Beeping Behavior Details

### Phase Change Timeline (Box Breathing Example)
```
Time  Event                      Audio
────────────────────────────────────────
0:00  Cycle 1 starts             (no beep, session starts)
0:04  INHALE → HOLD transition   🔊 BEEP
0:08  HOLD → EXHALE transition   🔊 BEEP
0:12  EXHALE → HOLD transition   🔊 BEEP
0:16  HOLD → INHALE (next cycle) 🔊 BEEP
0:20  Cycle 2 starts             (repeats)
```

### Sound Characteristics
- **Duration**: ~100ms short beep
- **Frequency**: Standard system beep (usually 1000 Hz)
- **Volume**: Respects system audio level
- **Timing**: Immediate (background thread)

## Keyboard Controls Summary

### All Screens
- `b` - Toggle beeping 🔊 ON / 🔇 OFF

### Menu
- `k/↑` `j/↓` - Navigate
- `Enter` - Select
- `h` - History
- `q` - Quit

### Setup
- `Tab` - Switch field
- `+/-` - Adjust
- `Enter` - Start
- `Esc` - Back

### Session
- `p/Space` - Pause/Resume
- `b` - Toggle beeping
- `e` - End early
- `q` - Quit

### Results & History
- `s` - Save (results only)
- Arrow keys - Navigate
- `Esc` - Back

## Feature Summary

✅ **Easy to Use** - Single `[b]` key toggles beeping  
✅ **Clear Status** - 🔊 (ON) or 🔇 (OFF) indicator  
✅ **Non-blocking** - Doesn't interrupt breathing  
✅ **Reliable** - Works on all platforms  
✅ **Optional** - Disable if not needed  
✅ **Fast** - Background threads, zero UI lag  
✅ **No Dependencies** - Uses only Rust stdlib  

Enjoy guided breathing with audio feedback! 🎵🧘‍♂️
