# SPEC: recompute fractal at pane resolution on resize

## Goal

Make the fractal panes fill the available window/pane area: when the
dock tab is resized (including when embedded in a resized browser
iframe), recompute the fractal at the new pixel resolution so the image
fits tightly with no clipping (pane too small) and no empty dock space
(pane too large). This mirrors how the native app feels when launched at
a matching window size.

## Background (current behavior)

The layout is bottom-up: grid resolution -> texture size -> table sizing.

- `MainInterface::show` (`crates/gui/src/interface.rs:1103`) builds a
  `TableBuilder` whose parent column is `Column::exact(image width)`,
  child column is `Column::remainder()`, and the image row height is the
  image height. Both planes share one `image_height`
  (`interface.rs:100`).
- `ImageFrame::show` (`crates/gui/src/image_frame.rs:82`) draws via
  `ui.image(texture)` at the texture's native pixel size; `region` is
  sized to the texture (`:117`). So **display size == grid resolution**
  by construction.
- `ui.available_size()` / pane rect is never read anywhere.
- Resolution lives in `PointGrid { res_x, res_y, bounds }`
  (`crates/common/src/point_grid.rs`); `res_x`/`res_y` are coupled via
  `infer_height`/`infer_width` and the bounds aspect ratio.
  `resize_y(h)` sets height and derives width from the bounds aspect.
- Compute is progressive (mip pyramid from `COARSEST_DIM=32` up) with a
  generation/cancel token; a new `submit` cancels any in-flight job
  (`crates/gui/src/compute.rs`). Coarse levels appear quickly.

Net: the fractal always renders at a fixed `image_height` (default 768,
`globals.rs:16`), so a larger dock pane shows empty space and a smaller
one clips.

## Design

### Preserve the 1:1 display==grid invariant

The safe approach is to **drive the grid resolution from the available
pane size**, keeping `display size == grid resolution`. This avoids
threading a `display/res` scale factor through every coordinate-mapping
site (`map_pixel`/`map_pos`/`map_vec2`, `locate_point`/`to_global_coords`,
`frame_contains_pixel`), which is the highest-risk alternative.

### Observe available size in `show`

In `MainInterface::show` (`interface.rs:1103`), read the available area
for the image row **before** building the `TableBuilder`
(`ui.available_size()` / `ui.available_height()`), and translate it to a
target `image_height`:

- The image row height maps directly to `image_height` (minus the header
  ~20px and the state-info row ~80px already in the layout).
- The two planes sit side by side (parent `exact`, child `remainder`),
  so each gets ~half the width. The target resolution is computed as a
  contain-fit of each plane into its available column box (width and
  height) given the plane's bounds aspect, so the image fills the column
  tightly regardless of the profile's aspect ratio. Width follows from
  `resize_y` -> `infer_width` (or `resize_x` -> `infer_height`, whichever
  axis is the binding constraint for the contain-fit).

### Debounce

Resizing by dragging changes the size every frame. An OS/browser window
resize is expected to be a cheap operation, so we must **not** kick off
fractal recomputes while a resize is in progress -- doing so could
saturate the CPU mid-drag. Requirements:

- **No compute during an active resize.** While the observed pane size is
  still changing, do nothing but record the latest target. Only after the
  size has been **stable** for a short interval (e.g. ~200 ms) do we flush
  a single recompute. The in-progress frames cost only a cheap
  integer-compare, never a `submit`.
- Track a `pending_height: Option<usize>` and a last-change timestamp on
  `MainInterface` (beside `image_height`). Each frame, compare the
  observed target against the current `image_height`; if it differs,
  update the pending value + timestamp (no compute). When `now - last`
  exceeds the debounce window and a pending value is still set, flush it
  once via the existing `change_height` (`interface.rs:1094`) ->
  `WindowPane::change_height` (`pane/mod.rs:671`) -> `grid.resize_y` +
  `schedule_compute`, then clear pending.
- Request a delayed repaint while a resize is pending
  (`ctx.request_repaint_after(debounce_window)`) so the flush fires even
  without further input, but otherwise add **no** continuous repaint.
- **Resolution cap.** Clamp the target resolution to a sane maximum
  (e.g. height <= ~2160, or a total-pixel budget) so maximizing on a 4K/5K
  display or a huge browser window cannot trigger a multi-megapixel
  fractal compute that overwhelms the CPU. Above the cap, fit by letterbox
  rather than computing more pixels.

### Profile changes alter aspect ratio

Different profiles carry different default view bounds, hence different
aspect ratios. Two consequences for this feature:

- **Re-fit on profile change, not just on pane-size change.** A profile
  switch can keep the pane the same size while changing the bounds aspect,
  so a trigger keyed only on pane-size delta would miss it. The fit must
  also re-run when the active plane's bounds/aspect change (e.g. after
  `set_child_param`, profile load, or a bounds reset). Practically: derive
  the target from the current available size every frame and compare
  against the *grid's* current `(res_x, res_y)` (which encodes the aspect),
  not just a cached height, so an aspect change registers as a needed
  re-fit and goes through the same debounce.
- **This pushes toward fitting both axes, not height-only.** With
  height-only fit, a profile whose aspect differs from the pane's aspect
  leaves a width gap or overflows the column. To fit tightly across
  profiles, compute the target from the available *box* (width and
  height) and the plane's bounds aspect: choose the resolution that fills
  the column without overflowing (contain-fit), adjusting `res_y` so the
  derived `res_x = infer_width(res_y, bounds)` fits the available width,
  and vice versa. Resolves open question 1 in favor of box-fit.

### Minimum / rounding

- Enforce a sensible minimum height (e.g. >= `COARSEST_DIM`, realistically
  a few hundred px) so degenerate tiny panes don't thrash or divide by
  zero.
- Round to whole pixels; ignore sub-pixel jitter (only act on integer
  changes beyond a small threshold, e.g. >= 2px, to avoid oscillation).
- Respect `pixels_per_point` (DPI): `available_size` is in points;
  multiply by `ctx.pixels_per_point()` to get the device-pixel resolution
  the texture should match, so HiDPI screens render sharp.

## Scope (this sprint)

0. Add debounced pane-size observation to `MainInterface` (fields +
   logic in `show`/`update`).
1. Translate available size (+ DPI) to a target `image_height`, flush via
   the existing `change_height` path once stable.
2. Replace the fixed default-height startup feel: the initial size still
   comes from `IMAGE_HEIGHT`, but the first frame's available size takes
   over immediately.
3. Keep the manual height menu buttons (`fractal_tab.rs:155-171`)
   working. Auto-resize wins on any pane-size or aspect change; a manual
   set is a one-shot that the next fit overrides. The manual buttons are
   retained for now and slated for removal in a followup once auto-resize
   is proven (see Followup).

## Out of scope

- Scaling the texture independently of grid resolution (rejected: breaks
  the 1:1 coordinate invariant).
- Independent per-pane (parent vs child) resolutions; keep the shared
  `image_height`.
- Website-side changes; the iframe already tracks the browser window via
  `vw/vh`, so once dynamo fits its pane, the embed fits the iframe.

## Verification

- `cargo check`/`clippy` for native and `wasm32-unknown-unknown`.
- Native: launch, drag-resize the window and dock splitter; confirm the
  fractal recomputes to fill the pane, sharp, with no clip/gap, and that
  drag-resize does not thrash (debounce holds).
- Coordinate sanity: after resize, mouse-to-plane mapping and marked
  points/curves still land correctly (the 1:1 invariant should keep these
  correct, but verify since resolution changed).
- Browser (the real target): rebuild the bundle, serve, confirm the
  embedded viewer fills the iframe and reflows when the browser window
  resizes.

## Rollout

After dynamo verification: commit + push to GitHub, then in the website
repo re-pin `vendor/dynamo`, rebuild the bundle, and redeploy. The
`/dynamo` page CSS already gives the iframe a large `96vw x 85vh` box, so
no website layout change is needed -- dynamo will now fill it.

## Open questions for review

0. Debounce mechanism on wasm: `std::time::Instant` panics on
   `wasm32-unknown-unknown`. Use egui's input time
   (`ctx.input(|i| i.time)`) or a frame counter instead. Confirm
   preference.
1. Contain-fit handling for the two unequal columns (parent `exact` vs
   child `remainder`): both planes share one `image_height`, so confirm
   fitting to the smaller-constraint column (so neither overflows) is
   acceptable, vs. letting the parent column width follow its own image.

## Followup (after this sprint, if auto-resize proves out)

Remove the manual image-height menu buttons (`fractal_tab.rs:155-171`)
and the now-redundant `change_height` plumbing, leaving auto-resize as
the sole sizing path.
