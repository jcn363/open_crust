# Development Guide

## Mission Control Feature

### Overview
The Mission Control view provides a real‑time visual dashboard for orchestrated multi‑agent workflows. It displays a directed acyclic graph (DAG) of tasks, live token/cost metrics per agent, and interactive controls.

### Architecture
- **UI Layer**: `ui/tabs/mission_control_tab.rs` implements the `Tab` trait and registers with the main UI dispatcher.
- **Components**:
  - `ui/components/graph.rs` – renders the DAG using Ratatui's `Canvas`.
  - `ui/components/metrics.rs` – shows per‑agent statistics.
  - `ui/components/controls.rs` – hotkey handling for pause/cancel/retry.
- **Orchestrator**: `orchestrator/coordinator.rs` now emits `OrchestratorEvent` and accepts `ControlCommand` via a `crossbeam::channel`.

### Key Files
- `src/orchestrator/coordinator.rs`
- `src/ui/tabs/mission_control_tab.rs`
- `src/ui/components/graph.rs`
- `src/ui/components/metrics.rs`
- `src/ui/components/controls.rs`

### Build & Test
```bash
cargo test --lib orchestrator --lib ui
```

### Usage
- Launch with `opencrust --mission-control`.
- Toggle with `Ctrl+Shift+M` from any tab.
- Hotkeys: `p` pause/resume, `c` cancel, `r` retry failed, `q` quit.

### Documentation
- Updated `README.md` with screenshots.
- Added `docs/mission_control.md` for detailed usage.
