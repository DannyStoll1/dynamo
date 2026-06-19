//! Off-thread, progressive fractal computation.
//!
//! A [`ComputeService`] runs each compute job on a shared global pool, walking a
//! mip pyramid from coarsest to finest. Each finished row chunk is colored on
//! the worker and streamed back as a [`Tile`], so the UI can fill the canvas
//! progressively without blocking. Submitting a new job cancels any in-flight
//! one via a generation token, and the worker stops promptly at chunk
//! boundaries.

use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::OnceLock;

use crossbeam::channel::{Receiver, Sender};
use dynamo_color::Coloring;
use dynamo_common::prelude::*;
use dynamo_core::prelude::*;
use egui::Color32;
#[cfg(not(target_arch = "wasm32"))]
use rayon::{ThreadPool, ThreadPoolBuilder};

/// Smallest mip level dimension, in pixels, along the shorter axis.
const COARSEST_DIM: usize = 32;

/// Bounded capacity for the tile channel. Large enough to absorb a burst of
/// chunk completions between UI frames without forcing workers to block often.
const TILE_CAPACITY: usize = 256;

/// A rectangular block of freshly colored pixels at one mip level.
///
/// `level_res` is the resolution of the level the tile belongs to, and `scale`
/// is `target_res / level_res` (approximately a power of two). The UI sizes the
/// display buffer to `target_res` and expands each tile pixel to a `scale x
/// scale` block (clamped to the buffer) so coarse levels cover the whole canvas
/// without changing the canvas size between levels.
pub struct Tile
{
    pub generation: u64,
    pub scale:      usize,
    pub level_res:  (usize, usize),
    /// Resolution of the finest (target) level; the display buffer size.
    pub target_res: (usize, usize),
    /// Pixel column range within the level (`x0..x1`).
    pub x_range:    (usize, usize),
    /// Pixel row range within the level (`y0..y1`), measured from the top.
    pub y_range:    (usize, usize),
    /// Row-major colors for the region, `(x1 - x0) * (y1 - y0)` entries.
    pub pixels:     Vec<Color32>,
}

/// A streamed update from an in-flight compute job.
pub enum Update<D>
{
    /// A freshly colored block ready to blit into the display buffer.
    Tile(Tile),
    /// The fully computed finest-level plane, emitted once when the job
    /// finishes. The pane caches it so palette changes can recolor without
    /// recomputing orbits.
    Done
    {
        generation: u64,
        plane:      Arc<IterPlane<D>>,
    },
}

/// The shared pool that hosts compute jobs.
///
/// One extra thread drives each job's mip walk; the data-parallel per-level
/// work fans out across rayon's global pool, preserving full parallelism and
/// SIMD within a level.
#[cfg(not(target_arch = "wasm32"))]
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

/// Run a compute job's body.
///
/// On native targets the body is spawned onto the shared compute pool so the
/// UI thread stays responsive while the mip walk progresses. On wasm there is
/// no thread support, so the body runs synchronously on the caller; tiles are
/// delivered via a non-blocking `try_send`, so this cannot deadlock, but the
/// caller blocks until the job completes.
fn dispatch<F>(body: F)
where
    F: FnOnce() + Send + 'static,
{
    #[cfg(not(target_arch = "wasm32"))]
    job_pool().spawn(body);
    #[cfg(target_arch = "wasm32")]
    body();
}

/// Immutable context shared across a single job's mip walk.
struct Job<'a, D>
{
    coloring:   &'a Coloring,
    token:      &'a CancelToken,
    generation: u64,
    target:     (usize, usize),
    updates_tx: &'a Sender<Update<D>>,
}

impl<D> Job<'_, D>
where
    D: Polar<Real> + Clone + Send + Sync + 'static,
{
    /// Walk the mip pyramid for `plane`, streaming colored tiles until done or
    /// cancelled. On completion, emit the finest plane for later recoloring.
    fn run<P>(&self, plane: &P)
    where
        P: Computable<Deriv = D> + Clone,
    {
        let pyramid = plane.point_grid().mip_pyramid(COARSEST_DIM);
        let finest = pyramid
            .last()
            .map_or((1, 1), |grid| (grid.res_x, grid.res_y));
        let mut level_plane = plane.clone();
        let mut prev: Option<IterPlane<D>> = None;

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
                scale: (finest.0 / level_grid.res_x.max(1)).max(1),
                res:   (level_grid.res_x, level_grid.res_y),
            };
            level_plane.compute_into_streaming(
                &mut iter_plane,
                &ChunkSink::streaming(
                    self.token,
                    &|x, y| skip_scale.is_some_and(|s| IterPlane::<D>::lifted_pixel(x, y, s)),
                    &|y_start, chunk| level.send_chunk(y_start, chunk),
                ),
            );

            if self.token.is_cancelled() {
                return;
            }
            prev = Some(iter_plane);
        }

        if let Some(finest) = prev {
            self.send(Update::Done {
                generation: self.generation,
                plane:      Arc::new(finest),
            });
        }
    }

    /// Recolor an already-computed plane, streaming tiles in row-chunk bands so
    /// the UI updates progressively, then re-emit the plane.
    fn recolor(&self, plane: &Arc<IterPlane<D>>)
    {
        let (res_x, res_y) = plane.point_grid.shape();
        let band = (res_y / num_cpus::get()).max(1);
        let mut y = 0;
        while y < res_y {
            if self.token.is_cancelled() {
                return;
            }
            let height = band.min(res_y - y);
            let chunk = plane.iter_counts.slice(ndarray::s![.., y..y + height]);
            let level = LevelStream {
                job:   self,
                scale: 1,
                res:   (res_x, res_y),
            };
            level.send_chunk(y, chunk);
            y += height;
        }

        self.send(Update::Done {
            generation: self.generation,
            plane:      Arc::clone(plane),
        });
    }

    /// Send an update, ignoring a full or disconnected channel: a later level
    /// supersedes dropped tiles, and a gone receiver means the job is moot.
    fn send(&self, update: Update<D>)
    {
        // Ignore a full or disconnected channel: a later level supersedes
        // dropped tiles, and a gone receiver means the job is moot.
        let _ = self.updates_tx.try_send(update);
    }
}

/// Per-level streaming parameters, used to color and dispatch finished chunks.
struct LevelStream<'a, D>
{
    job:   &'a Job<'a, D>,
    scale: usize,
    res:   (usize, usize),
}

impl<D> LevelStream<'_, D>
where
    D: Polar<Real> + Clone + Send + Sync + 'static,
{
    /// Color a finished chunk and push it as a tile. The chunk view is shaped
    /// `(res_x, chunk_height)` and corresponds to rows
    /// `y_start..y_start + height`.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "ArrayView2 is a Copy borrow handle, so taking it by value is zero-cost and matches the streaming callback signature"
    )]
    fn send_chunk(&self, y_start: usize, chunk: ndarray::ArrayView2<PointInfo<D>>)
    {
        let (res_x, height) = (chunk.shape()[0], chunk.shape()[1]);
        let mut pixels = Vec::with_capacity(res_x * height);
        // Row-major, top to bottom, matching the UI blit.
        for local_y in 0..height {
            for x in 0..res_x {
                pixels.push(self.job.coloring.map::<D, Color32>(&chunk[(x, local_y)]));
            }
        }

        self.job.send(Update::Tile(Tile {
            generation: self.job.generation,
            scale: self.scale,
            level_res: self.res,
            target_res: self.job.target,
            x_range: (0, res_x),
            y_range: (y_start, y_start + height),
            pixels,
        }));
    }
}

/// Per-pane handle for submitting progressive compute jobs and draining updates.
pub struct ComputeService<D>
{
    cancel:     CancelSource,
    updates_rx: Receiver<Update<D>>,
    updates_tx: Sender<Update<D>>,
    in_flight:  bool,
}

impl<D> Default for ComputeService<D>
{
    fn default() -> Self
    {
        let (updates_tx, updates_rx) = crossbeam::channel::bounded(TILE_CAPACITY);
        Self {
            cancel: CancelSource::new(),
            updates_rx,
            updates_tx,
            in_flight: false,
        }
    }
}

impl<D> ComputeService<D>
where
    D: Polar<Real> + Clone + Send + Sync + 'static,
{
    #[must_use]
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Whether a job is currently producing updates.
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
    /// job. Updates stream back coarsest level first; drain them with
    /// [`drain`](Self::drain).
    pub fn submit<P>(&mut self, plane: Arc<P>, coloring: Arc<Coloring>)
    where
        P: Computable<Deriv = D> + Clone + Send + Sync + 'static,
    {
        let token = self.cancel.renew();
        let generation = token.generation();
        let updates_tx = self.updates_tx.clone();
        self.in_flight = true;

        dispatch(move || {
            let grid = plane.point_grid();
            let job = Job {
                coloring: coloring.as_ref(),
                token: &token,
                generation,
                target: (grid.res_x, grid.res_y),
                updates_tx: &updates_tx,
            };
            job.run(plane.as_ref());
        });
    }

    /// Recolor an already-computed finest plane without recomputing orbits,
    /// streaming fresh tiles for the new coloring. Used for palette changes.
    pub fn recolor(&mut self, plane: Arc<IterPlane<D>>, coloring: Arc<Coloring>)
    {
        let token = self.cancel.renew();
        let generation = token.generation();
        let updates_tx = self.updates_tx.clone();
        self.in_flight = true;

        dispatch(move || {
            let job = Job {
                coloring: coloring.as_ref(),
                token: &token,
                generation,
                target: (plane.point_grid.res_x, plane.point_grid.res_y),
                updates_tx: &updates_tx,
            };
            job.recolor(&plane);
        });
    }

    /// Drain every update produced since the last call, dropping any from a
    /// stale generation and clearing the busy flag when the job completes.
    pub fn drain(&mut self) -> Vec<Update<D>>
    {
        let current = self.cancel.generation();
        let mut updates = Vec::new();
        for update in self.updates_rx.try_iter() {
            let (generation, done) = match &update {
                Update::Tile(tile) => (tile.generation, false),
                Update::Done { generation, .. } => (*generation, true),
            };
            if generation != current {
                continue;
            }
            if done {
                self.in_flight = false;
            }
            updates.push(update);
        }
        updates
    }
}
