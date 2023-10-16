## About

A powerful tool for studying complex dynamics. Inspired heavily by the following projects:

- Wolf Jung's [Mandel](https://mndynamics.com/indexp.html) (many of whose hotkeys are intentionally reused)
- Brian and Susanna Boyd's [Dynamics Explorer](https://sourceforge.net/projects/detool/)
- Matt Noonan's [FractalStream](https://pi.math.cornell.edu/~noonan/fstream.html)
  Fractal Explorer hopes to combine the strengths of these excellent tools, though it is currently in a very early stage.

## Features

- Over 100 built-in profiles for commonly studied dynamical systems
- Live Julia sets
- "Meta-parameter planes" (e.g. multiplier plane for Cubic Per(1, λ)) with live views of the child planes
- Live tracking for critical points and cycles
- Smooth coloring for both escaping and non-escaping components
- Period coloring
- External rays (working for quadratic polynomials and some other families, unstable in general)
- Equipotentials (not fully reliable, working on a better implementation)
- Optimized for performance

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
- Shift+Space: Reset selection

### Dynamics

- F: Apply map to selection [dynamical plane]
- Ctrl-F: Find parameter of given preperiod/period near selection [active plane]
- E: External ray [active plane]
- Ctrl-X: External ray to point [active plane]

### Computation

- +: Increase max iters
- -: Decrease max iters
- Ctrl-S: save image (prompt in command line; currently does not include marked points/curves)
- L: Toggle Live Julia mode (update the child plane as the cursor moves in the parent plane)

### Annotations

- I: Toggle selection [active plane]
- Ctrl-\<N\>: Toggle cycles of period \<N\>, if they are implemented for the given system [dynamical plane]
- Ctrl-Shift-\<N\>: Toggle component centers of period \<N\>, if they are implemented for the given system [parameter plane]
- O: Toggle marked points [parameter plane]
- P: Toggle critical points [dynamical plane]
- C: Clear orbit
- Shift-C: Clear all marked curves

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
  - [x] User-friendly save dialog
- [ ] Buttons for all actions
- [ ] Command-line integration
- [x] Internal coloration
- [ ] Drag to pan/zoom
- [ ] Descend to child for multi-parameter systems
- [ ] Solve for critical points and $n$-cycles automatically
- [ ] User-friendly scripting interface
- [ ] Switch to stable channel
- [x] Implement web interface
  - [x] Fix broken clicking in web UI
  - [x] Fix slow initial rendering in web UI
