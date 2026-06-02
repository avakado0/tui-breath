# Session Storage Schema

`tui_breath` stores each completed or abandoned Session as JSON plus a lightweight history index for fast browsing.

## Session Record

```json
{
  "session_id": "uuid",
  "start_time": "2026-06-02T12:00:00Z",
  "end_time": "2026-06-02T12:03:10Z",
  "status": "completed",
  "type": "breathing",
  "parameters": {
    "duration_target": 180,
    "actual_duration_secs": 150.4,
    "settings": {
      "rate": 8.0,
      "phase_parameters": {
        "inhalation_time": 4.0,
        "exhalation_time": 8.0,
        "hold_in_time": 7.0,
        "hold_out_time": null
      },
      "iterations": 18,
      "pattern_id": "478",
      "tempo": 1.0
    }
  },
  "breath_hold": {
    "best_seconds": 23.1,
    "attempt_count": 2,
    "attempts": [
      {
        "started_at": "2026-06-02T12:01:00Z",
        "ended_at": "2026-06-02T12:01:18Z",
        "duration_secs": 18.0
      },
      {
        "started_at": "2026-06-02T12:02:00Z",
        "ended_at": "2026-06-02T12:02:23Z",
        "duration_secs": 23.1
      }
    ]
  },
  "history": [
    {
      "timestamp": "2026-06-02T12:00:00Z",
      "event": "Start",
      "details": { "details": "Session started: 4-7-8 Breathing at tempo 1" }
    },
    {
      "timestamp": "2026-06-02T12:01:00Z",
      "event": "HoldStart",
      "details": { "details": "Breath hold started at 60.0s" }
    }
  ]
}
```

## History Index Entry

```json
{
  "session_id": "uuid",
  "start_time": "2026-06-02T12:00:00Z",
  "status": "completed",
  "pattern_id": "478",
  "duration_target": 180,
  "cycles_completed": 18,
  "completion_pct": 83.6,
  "best_breath_hold_seconds": 23.1,
  "breath_hold_attempt_count": 2
}
```

## Notes

- `breath_hold` is optional so older Session files remain valid.
- `best_breath_hold_seconds` and `breath_hold_attempt_count` default cleanly when older index entries are loaded.
- `actual_duration_secs` reflects active breathing time only. Breath Hold time is tracked separately and does not inflate breathing completion.
