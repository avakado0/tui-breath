# Breathing App Market Research — Inspiration for tui_breath

Date: 2026-05-02
Source: Web search of top breathing apps (Breathwrk, Inhale, Bretho, BreatheX, BreathWave, Calm, Insight Timer, etc.)

---

## Apps Reviewed

| App | Core Identity | Standout Features |
|-----|--------------|-----------------|
| **Breathwrk** | Goal-based breathwork (Sleep, Stress, Focus, Energy) | 100+ exercises, curated **Habits** (daily routines), **Classes** (coach-led), haptic vibration, custom visuals/sounds, 7-day challenges |
| **Inhale** | Guided timer with progress tracking | **Breathing circle** animation, **streaks**, pre/post **mood tracking**, **CO2 tolerance** measurement, partner breathing |
| **Bretho** | AI + custom sessions | **AI bot** generates sessions from mood, **custom pattern builder** (precise inhale/hold/exhale control), voice packs, soundscapes |
| **BreatheX** | Routine-focused | Browse routines by category, **create custom routines**, **daily minute goals**, **reminders**, practice logs |
| **BreathWave** | Professional training | 6 progressive **difficulty levels**, standalone watch mode, **dynamic visual guides** + ambient noise |
| **Yogi Breath** | Pranayama tradition | 42 techniques across **6 levels**, structured progression from beginner → advanced |
| **Breah / Mindful Breathing** | Minimalist habit | **Mindful Goals** (daily targets), binaural backgrounds, voice guides, no paywall |
| **Vayu** | Hardware-integrated | **HRV biofeedback** (smartwatch), haptic guidance synced to breath |
| **Wim Hof Method** | Breathwork + cold exposure training | **Super-ventilation** (30–40 deep breaths), breath-hold retention, guided cold prep, structured progress tracking |
| **Calm** | All-in-one wellness | Sleep stories, meditation, **breathing programs**, personalized content |
| **Insight Timer** | Library model | Thousands of free tracks, community teachers, open marketplace |

---

## What Our TUI Already Has

From `CLAUDE.md` and the current codebase:

- **Pattern library** — 3 built-in patterns (4-7-8, Box, Diaphragmatic)
- **Session customization** — duration + tempo
- **Visual guide** — breathing bar (expand/contract with phase)
- **Audio cue** — beep on phase change (terminal bell)
- **Pause/resume** — during session
- **Post-session metrics** — cycles, pauses, completion %
- **History** — browsable past sessions with JSON storage

---

## Inspirational Features for a TUI

Organized by effort, with the GUI app that inspired each.

### Quick Wins (Terminal-Native)

| Feature | App Source | TUI Equivalent |
|---------|-----------|----------------|
| **Quick SOS sessions** | Breathwrk, Breethe | 1–2 minute presets (skip setup screen) |
| **Pattern categories** | Breathwrk | Filter menu: Sleep / Calm / Focus / Energy |
| **Pre/post mood check-in** | Inhale | Simple 1–5 rating prompt before/after session |
| **Session notes** | Insight Timer | Free-text input in Results screen |
| **Daily streak counter** | Inhale, Breah | Show streak in menu screen header |
| **Favorite patterns** | Breathwrk | Mark patterns, show "Favorites" section first |

### Medium Features

| Feature | App Source | TUI Equivalent |
|---------|-----------|----------------|
| **Custom pattern builder** | Bretho, BreatheX | New "Create Pattern" screen: set inhale/hold/exhale durations, name it, save to JSON |
| **More built-in patterns** | BreathWave, Yogi Breath, **Wim Hof** | Add Equal Breathing, 4-4-4-4 (Box variant), Relaxing Breath, **Wim Hof technique** (30–40 deep breaths + breath hold retention) |
| **Program / challenges** | Breathwrk | 7-day sequence file (Day 1: Box, Day 2: 4-7-8, etc.) |
| **Better stats** | Inhale | Weekly summary screen: sessions, total minutes, consistency graph (ASCII sparkline) |
| **Daily reminders** | BreatheX | Config file setting (`remind_at: "09:00"`) — Pi could nudge |

### Stretch / Later

| Feature | App Source | TUI Equivalent |
|---------|-----------|----------------|
| **HRV biofeedback** | Vayu, Inhale | Would need hardware integration (probably out of scope) |
| **Ambient sound** | Breathwrk, Breah | Requires audio dependency; could delegate to system |
| **Difficulty levels** | Yogi Breath, BreathWave | Tag patterns: Beginner / Intermediate / Advanced |
| **Partner breathing** | Inhale | Network sync — way out of scope |

---

## Most Inspiring for Our TUI

1. **Breathwrk's "Habits"** → A **program mode**: pick a goal, get a pre-sequenced multi-day plan displayed in the menu.
2. **Bretho's custom patterns** → A **pattern builder** is the single biggest feature gap vs. GUI apps.
3. **Inhale's streaks + mood** → Adds **user motivation** without adding UI complexity.
4. **BreatheX's daily goals** → Simple config: "I want 10 minutes/day" → progress shown on menu screen.

---

## Sources

- [Top 10 Breathing Apps 2026](https://breathworkk.app/blog/top-breathing-apps-2026)
- [Breathwrk App Review](https://www.choosingtherapy.com/breathwrk-app-review/)
- [Inhale App](https://getinhale.app/)
- [Bretho App](https://www.bretho.app/)
- [BreatheX](https://www.breathex.io/)
- [BreathWave](http://breath-wave.com/)
- [Best Breathing App Without Subscription](https://undulate.app/blog/best-breathing-app-no-subscription)
- [Mindful Suite Reviews](https://www.mindfulsuite.com/reviews/best-breathing-exercise-apps)
