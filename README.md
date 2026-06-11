# bounce

> A three-panel cyberpunk screensaver / mini-game, fully driven by live system data.

The most meta of the 10 scenes: a three-panel layout where the left panel is a live diagnostics readout, the middle is a simulated command console, and the right is a side-scrolling bunny-hop game whose AI skill level adapts over time.

## Panels

1. **SYSTEM DIAGNOSTICS** (left). Rich live readout:
   - Hostname + username
   - OS name + build (live, with proper Win11 detection)
   - Kernel version
   - Uptime
   - Shell
   - Display resolution + refresh rate
   - CPU
   - Memory usage (with percentage)
   - Theme + accent color (with hex)
   - Power status
   - Disk summary
   - GPU(s) and Monitor(s)
   - AI skill percentage (see below)

2. **COMMAND CONSOLE** (middle). Simulated typing of realistic system commands with output. Feels like watching a terminal session.

3. **BUNNY HOP** (right). A side-scrolling bunny-hopping game.

## Dynamic / live behavior

- Pulls full `SystemInfo` (uptime, memory, power, disk, GPUs, monitors).
- Stats refresh every ~1 second.
- The bunny has an **adaptive AI skill system** (0.72 → 0.98). In autonomous mode it gradually gets good at the game — better jump timing, higher success rate, more consistent runs as time passes or after good scores. Crashes cause slight skill decay.
- Press **Space** while the screensaver is running to manually jump (helps the AI learn and boosts score/speed). Other inputs still exit normally.
- `host_bias` and load reactions influence the overall feel.

## Configuration (registry)

Under `HKEY_CURRENT_USER\Software\local76\bounce`:

- `Speed`: 0 = slow, 1 = normal, 2 = fast.
- `ShowSysInfo`: 0 = hide left panel, 1 = show full diagnostics.

## Notes

- This is the meta scene of the collection — it literally shows you information about the machine it is running on.
- The bunny getting better over long runs is intentional and satisfying to watch.
- Fully playable while still functioning as a proper screensaver (only non-space inputs exit).

Part of the local76 screensaver collection. See [local76/local76](https://github.com/local76/local76) for the full list of scenes.
