## Installation

To install and run, just clone the repository, navigate to `bin`, and run `cargo +nightly run -r`.

You may need to first [install Rust](https://rustup.rs/). At present, the nightly toolchain is required; you can install this with `rustup install nightly`.

## Usage

### Navigation

- Z: zoom in to selection
- Ctrl-Z: zoom in far
- V: zoom out from selection
- Ctrl-V: zoom out far
- Shift+arrows: pan view
- Space: Center selection

### Computation

- +: Increase max iters
- -: Decrease max iters
- Ctrl-S: save image (prompt in command line; currently does not include marked points/curves)
- H: Resize images
- L: Toggle live mode
- Y: Toggle fixed points
- U: Toggle 2-cycles
- P: Toggle critical points
- C: Clear marked curves
- Shift+Space: Reset selection

### Coloring

- R: Randomize palette
- W: White palette
- B: Black palette
- 0: Internal coloration: Solid
- 1: Internal coloration: Period
- 2: Internal coloration: Period and Multiplier
- 3: Internal coloration: Multiplier
- 4: Internal coloration: Preperiod
- 5: Internal coloration: Potential of linearizing coordinate
- Up/Down: Change coloring period
- Right/Left: Change coloring phase

## Planned Features

- [x] Live Julia sets
- [x] Mark orbits
- [x] Marked points
- Saving improvements
  - [ ] Save/load palettes
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
- [x] Implement web interface
  - [x] Fix broken clicking in web UI
  - [x] Fix slow initial rendering in web UI
