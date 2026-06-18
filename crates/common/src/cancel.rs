//! Cooperative cancellation for long-running, chunked computations.
//!
//! A [`CancelSource`] hands out [`CancelToken`]s stamped with a generation. When
//! a new directive supersedes the old one, the source advances its generation,
//! which immediately marks every previously issued token as cancelled. Workers
//! poll [`CancelToken::is_cancelled`] at a coarse granularity (for example, once
//! per row chunk) and abandon stale work.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Issues [`CancelToken`]s tied to a monotonically increasing generation.
///
/// Cloning a source shares the same underlying generation counter, so a token
/// issued through any clone is cancelled once the generation advances.
#[derive(Clone, Debug, Default)]
pub struct CancelSource
{
    generation: Arc<AtomicU64>,
}

impl CancelSource
{
    #[must_use]
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Advance the generation, cancelling every previously issued token, and
    /// return a fresh token stamped with the new generation.
    #[must_use]
    pub fn renew(&self) -> CancelToken
    {
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        CancelToken {
            generation,
            latest: Arc::clone(&self.generation),
        }
    }

    /// Issue a token for the current generation without advancing it.
    #[must_use]
    pub fn token(&self) -> CancelToken
    {
        CancelToken {
            generation: self.generation.load(Ordering::SeqCst),
            latest:     Arc::clone(&self.generation),
        }
    }

    /// The current generation.
    #[must_use]
    pub fn generation(&self) -> u64
    {
        self.generation.load(Ordering::SeqCst)
    }
}

/// A handle that reports whether the work it guards has been superseded.
#[derive(Clone, Debug)]
pub struct CancelToken
{
    generation: u64,
    latest:     Arc<AtomicU64>,
}

impl CancelToken
{
    /// A token that is never cancelled. Useful for synchronous callers that do
    /// not participate in cancellation.
    #[must_use]
    pub fn never() -> Self
    {
        Self {
            generation: 0,
            latest:     Arc::new(AtomicU64::new(0)),
        }
    }

    /// Whether a newer generation has superseded this token.
    #[must_use]
    pub fn is_cancelled(&self) -> bool
    {
        self.latest.load(Ordering::Relaxed) != self.generation
    }

    /// The generation this token was stamped with.
    #[must_use]
    pub const fn generation(&self) -> u64
    {
        self.generation
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn renew_cancels_prior_tokens()
    {
        let source = CancelSource::new();
        let first = source.renew();
        assert!(!first.is_cancelled());

        let second = source.renew();
        assert!(first.is_cancelled());
        assert!(!second.is_cancelled());
    }

    #[test]
    fn never_token_stays_live()
    {
        let token = CancelToken::never();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn shared_across_clones()
    {
        let source = CancelSource::new();
        let clone = source.clone();
        let token = source.renew();
        // Advancing through the clone cancels a token issued through the original.
        let _superseding = clone.renew();
        assert!(token.is_cancelled());
    }
}
