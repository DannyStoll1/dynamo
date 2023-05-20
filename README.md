# Installation

To install and run, just clone the repository and run `cargo +nightly run -r`.

You may need to first [install Rust](https://rustup.rs/). At present, the nightly toolchain is required; you can install this with `rustup install nightly`.

# Usage

## Hotkeys

- Z: zoom in to selection
- Ctrl-Z: zoom in far
- V: zoom out from selection
- Ctrl-V: zoom out far
- Ctrl-S: save image (prompt in command line; currently does not include marked points/curves)
- H: Resize images
- L: Toggle live mode
- R: Randomize palette
- W: White palette
- B: Black palette
- Y: Toggle fixed points
- U: Toggle 2-cycles
- P: Toggle critical points
- +: Increase max iters
- -: Decrease max iters
- C: Clear marked curves
- 0: Internal coloration: Solid
- 1: Internal coloration: Period
- 2: Internal coloration: Period and Multiplier
- 3: Internal coloration: Multiplier
- 4: Internal coloration: Preperiod
- 5: Internal coloration: Potential of linearizing coordinate
- Up/Down: Change coloring rate
- Right/Left: Change coloring phase

# Planned Features
- [x] Live Julia sets
- [x] Mark orbits
- [x] Marked points
- [ ] Save palettes
- [x] Save images
- [ ] Marked points/curves in saved images
- [ ] Save program state
- [ ] User-friendly save dialog
- [ ] Buttons for all actions
- [ ] Command-line integration
- [x] Internal coloration
- [ ] Drag to pan/zoom
- [ ] Descend to child for multi-parameter systems
- [ ] Solve for critical points and $n$-cycles automatically
- [ ] User-friendly scripting interface
- [ ] Remove nightly requirement
