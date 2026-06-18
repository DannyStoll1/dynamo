//! Off-thread, progressive fractal computation.
//!
//! A [`ComputeService`] runs each compute job on a shared global pool, walking a
//! mip pyramid from coarsest to finest. Each finished row chunk is colored on
//! the worker and streamed back as a [`Tile`], so the UI can fill the canvas
//! progressively without blocking. Submitting a new job cancels any in-flight
//! one via a generation token, and the worker stops promptly at chunk
//! boundaries.

use std::sync::{Arc, OnceLock};

use crossbeam::channel::{Receiver, Sender, TrySendError};
use dynamo_color::Coloring;
use dynamo_common::prelude::*;
use dynamo_core::prelude::*;
use egui::Color32;
use rayon::{ThreadPool, ThreadPoolBuilder};

/// Smallest mip level dimension, in pixels, along the shorter axis.
const COARSEST_DIM: usize = 32;

/// Bounded capacity for the tile channel. Large enough to absorb a burst of
/// chunk completions between UI frames without forcing workers to block often.
const TILE_CAPACITY: usize = 256;

/// A rectangular block of freshly colored pixels at one mip level.
///
/// `level_res` is the resolution of the level the tile belongs to, and `scale`
/// is `target_res / level_res` (a power of two). The UI expands each tile pixel
/// to a `scale x scale` block so coarse levels cover the whole canvas.
pub struct Tile
{
    pub generation: u64,
    pub scale:      usize,
    pub level_res:  (usize, usize),
    /// Pixel column range within the level (`x0..x1`).
    pub x_range:    (usize, usize),
    /// Pixel row range within the level (`y0..y1`), measured from the top.
    pub y_range:    (usize, usize),
    /// Row-major colors for the region, `(x1 - x0) * (y1 - y0)` entries.
    pub pixels:     Vec<Color32>,
}

/// The shared pool that hosts compute jobs.
///
/// One extra thread drives each job's mip walk; the data-parallel per-level
/// work fans out across rayon's global pool, preserving full parallelism and
/// SIMD within a level.
fn job_pool() -> &'static ThreadPool
{
    static POOL: OnceLock<ThreadPool> = OnceLock::new();
    POOL.get_or_init(|| {
        ThreadPoolBuilder::new()
            .num_threads(2)
            .thread_name(|i| format!("dynamo-compute-{i}"))
            .build()
            .expect("failed to build compute pool")
    })
}

/// Immutable context shared across a single job's mip walk.
struct Job<'a>
{
    coloring:   &'a Coloring,
    token:      &'a CancelToken,
    generation: u64,
    finest_res: usize,
    tiles_tx:   &'a Sender<Tile>,
}

impl Job<'_>
{
    /// Walk the mip pyramid for `plane`, streaming colored tiles until done or
    /// cancelled.
    fn run<P>(&self, plane: &P)
    where
        P: Computable + Clone,
        P::Deriv: Polar<Real>,
    {
        let pyramid = plane.point_grid().mip_pyramid(COARSEST_DIM);
        let mut level_plane = plane.clone();
        let mut prev: Option<IterPlane<P::Deriv>> = None;

        for level_grid in pyramid {
            if self.token.is_cancelled() {
                return;
            }
            level_plane.point_grid_mut().clone_from(&level_grid);

            let mut iter_plane = IterPlane::create(level_grid.clone());
            let skip_scale = prev
                .as_ref()
                .and_then(|coarse| iter_plane.lift_from_coarser(coarse));

            let level = LevelStream {
                job:   self,
                scale: self.finest_res / level_grid.res_x.max(level_grid.res_y).max(1),
                res:   (level_grid.res_x, level_grid.res_y),
            };
            level_plane.compute_into_streaming(
                &mut iter_plane,
                &ChunkSink::streaming(
                    self.token,
                    &|x, y| {
                        skip_scale.is_some_and(|s| IterPlane::<P::Deriv>::lifted_pixel(x, y, s))
                    },
                    &|y_start, chunk| level.send_chunk(y_start, chunk),
                ),
            );

            if self.token.is_cancelled() {
                return;
            }
            prev = Some(iter_plane);
        }
    }
}

/// Per-level streaming parameters, used to color and dispatch finished chunks.
struct LevelStream<'a>
{
    job:   &'a Job<'a>,
    scale: usize,
    res:   (usize, usize),
}

impl LevelStream<'_>
{
    /// Color a finished chunk and push it as a tile. The chunk view is shaped
    /// `(res_x, chunk_height)` and corresponds to rows
    /// `y_start..y_start + height`.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "ArrayView2 is a Copy borrow handle, so taking it by value is zero-cost and matches the streaming callback signature"
    )]
    fn send_chunk<D>(&self, y_start: usize, chunk: ndarray::ArrayView2<PointInfo<D>>)
    where
        D: Polar<Real>,
    {
        let (res_x, height) = (chunk.shape()[0], chunk.shape()[1]);
        let mut pixels = Vec::with_capacity(res_x * height);
        // Row-major, top to bottom, matching the UI blit.
        for local_y in 0..height {
            for x in 0..res_x {
                pixels.push(self.job.coloring.map::<D, Color32>(&chunk[(x, local_y)]));
            }
        }

        let tile = Tile {
            generation: self.job.generation,
            scale: self.scale,
            level_res: self.res,
            x_range: (0, res_x),
            y_range: (y_start, y_start + height),
            pixels,
        };

        // Drop tiles rather than block the worker if the UI falls behind; a
        // later level will supersede them anyway.
        if let Err(TrySendError::Disconnected(_)) = self.job.tiles_tx.try_send(tile) {
            // Receiver is gone; nothing more to do.
        }
    }
}

/// Per-pane handle for submitting progressive compute jobs and draining tiles.
pub struct ComputeService
{
    cancel:    CancelSource,
    tiles_rx:  Receiver<Tile>,
    tiles_tx:  Sender<Tile>,
    in_flight: bool,
}

impl Default for ComputeService
{
    fn default() -> Self
    {
        let (tiles_tx, tiles_rx) = crossbeam::channel::bounded(TILE_CAPACITY);
        Self {
            cancel: CancelSource::new(),
            tiles_rx,
            tiles_tx,
            in_flight: false,
        }
    }
}

impl ComputeService
{
    #[must_use]
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Whether a job is currently producing tiles.
    #[must_use]
    pub const fn is_busy(&self) -> bool
    {
        self.in_flight
    }

    /// Cancel any in-flight job without starting a new one.
    pub fn cancel(&mut self)
    {
        let _superseding = self.cancel.renew();
        self.in_flight = false;
    }

    /// Submit a progressive compute over `plane`'s grid, cancelling any prior
    /// job. Tiles stream back coarsest level first; drain them with
    /// [`drain_tiles`](Self::drain_tiles).
    pub fn submit<P>(&mut self, plane: Arc<P>, coloring: Arc<Coloring>)
    where
        P: Computable + Clone + Send + Sync + 'static,
        P::Deriv: Polar<Real>,
    {
        let token = self.cancel.renew();
        let generation = token.generation();
        let tiles_tx = self.tiles_tx.clone();
        self.in_flight = true;

        job_pool().spawn(move || {
            let finest_res = plane.point_grid().res_x.max(plane.point_grid().res_y);
            let job = Job {
                coloring: coloring.as_ref(),
                token: &token,
                generation,
                finest_res,
                tiles_tx: &tiles_tx,
            };
            job.run(plane.as_ref());
        });
    }

    /// Drain every tile produced since the last call, dropping any from a stale
    /// generation. Returns tiles in arrival order.
    pub fn drain_tiles(&self) -> impl Iterator<Item = Tile> + '_
    {
        let current = self.cancel.generation();
        self.tiles_rx
            .try_iter()
            .filter(move |tile| tile.generation == current)
    }
}
