//! Opaque block storage. The broker stores **exact envelope bytes** keyed by the
//! random [`BlockId`] the client chose (§8.4: "store exact envelope bytes, make
//! idempotent operations stable"). It never parses past the envelope header and
//! never holds a key, so a stored block is inert to it.

use std::collections::{BTreeMap, BTreeSet};

use sparq_e2ee_ng::ids::{BlockId, TopicId};

/// One stored opaque block.
#[derive(Debug, Clone)]
pub struct StoredBlock {
    /// The canonical envelope bytes exactly as received.
    pub bytes: Vec<u8>,
    /// Topics this block is attributed to (a block put under two topics is
    /// retained while either retains it).
    pub topics: BTreeSet<TopicId>,
    /// Unix second of the last put/get that touched this block; drives
    /// age-based retention.
    pub last_touch: u64,
    /// `true` when the block is the root block of an announced commit; roots are
    /// the last thing evicted under pressure.
    pub is_commit_root: bool,
}

/// An in-memory opaque block store.
#[derive(Debug, Default)]
pub struct BlockStore {
    blocks: BTreeMap<BlockId, StoredBlock>,
}

/// What one garbage-collection pass removed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GcReport {
    /// Blocks removed.
    pub removed_blocks: u64,
    /// Bytes reclaimed.
    pub removed_bytes: u64,
}

impl BlockStore {
    /// Empty store.
    pub fn new() -> Self {
        BlockStore::default()
    }

    /// Number of stored blocks.
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Is the store empty?
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Store `bytes` under `id`, attributed to `topic`. Idempotent: a repeat put
    /// of the *same* id refreshes the touch time and topic attribution and
    /// returns `false` (already present) without overwriting the stored bytes.
    ///
    /// Not overwriting is deliberate. A block id is authenticated inside its
    /// encrypted parent (§4.1), so the first-seen bytes are the ones a reader's
    /// AEAD will accept; letting a later put replace them would hand any client
    /// that learns an id a way to make a peer's fetch fail closed. A client that
    /// receives bytes it cannot open re-fetches — the broker is not trusted for
    /// integrity in the first place (§5).
    pub fn put(&mut self, id: BlockId, bytes: Vec<u8>, topic: TopicId, now: u64) -> bool {
        match self.blocks.get_mut(&id) {
            Some(existing) => {
                existing.topics.insert(topic);
                existing.last_touch = now;
                false
            }
            None => {
                let mut topics = BTreeSet::new();
                topics.insert(topic);
                self.blocks.insert(
                    id,
                    StoredBlock {
                        bytes,
                        topics,
                        last_touch: now,
                        is_commit_root: false,
                    },
                );
                true
            }
        }
    }

    /// Is this block held **for `topic`**? Presence is answered only within the
    /// asking session's routing context, so the broker never confirms to one
    /// topic's peers that a block exists under another topic.
    pub fn contains_in(&self, id: &BlockId, topic: &TopicId) -> bool {
        self.blocks
            .get(id)
            .is_some_and(|b| b.topics.contains(topic))
    }

    /// Fetch the exact stored bytes for `id` within `topic`, refreshing its touch
    /// time.
    pub fn get_in(&mut self, id: &BlockId, topic: &TopicId, now: u64) -> Option<&[u8]> {
        let b = self.blocks.get_mut(id)?;
        if !b.topics.contains(topic) {
            return None;
        }
        b.last_touch = now;
        Some(&b.bytes)
    }

    /// Mark a block as an announced commit root.
    pub fn mark_commit_root(&mut self, id: &BlockId) {
        if let Some(b) = self.blocks.get_mut(id) {
            b.is_commit_root = true;
        }
    }

    /// Every block id attributed to `topic`, in ascending id order (the order the
    /// have/want pager walks).
    pub fn ids_in(&self, topic: &TopicId) -> Vec<BlockId> {
        self.blocks
            .iter()
            .filter(|(_, b)| b.topics.contains(topic))
            .map(|(id, _)| *id)
            .collect()
    }

    /// `(blocks, bytes)` currently attributed to `topic`.
    pub fn usage(&self, topic: &TopicId) -> (u64, u64) {
        self.blocks
            .values()
            .filter(|b| b.topics.contains(topic))
            .fold((0, 0), |(n, sz), b| (n + 1, sz + b.bytes.len() as u64))
    }

    /// Drop the topic attribution from every block, then remove blocks that no
    /// topic retains. Used when an epoch retires a topic and its storage is
    /// released.
    pub fn drop_topic(&mut self, topic: &TopicId) -> GcReport {
        for b in self.blocks.values_mut() {
            b.topics.remove(topic);
        }
        self.sweep_unattributed()
    }

    fn sweep_unattributed(&mut self) -> GcReport {
        let mut report = GcReport::default();
        self.blocks.retain(|_, b| {
            if b.topics.is_empty() {
                report.removed_blocks += 1;
                report.removed_bytes += b.bytes.len() as u64;
                false
            } else {
                true
            }
        });
        report
    }

    /// Age-based collection: remove blocks whose every attributed topic is
    /// unpinned and whose last touch is at least `ttl` seconds old.
    /// `is_pinned` reports a topic's pin state.
    pub fn collect_aged(
        &mut self,
        now: u64,
        ttl: u64,
        is_pinned: &dyn Fn(&TopicId) -> bool,
    ) -> GcReport {
        if ttl == u64::MAX {
            return GcReport::default();
        }
        let mut report = GcReport::default();
        self.blocks.retain(|_, b| {
            let pinned = b.topics.iter().any(is_pinned);
            let aged = now.saturating_sub(b.last_touch) >= ttl;
            if !pinned && aged {
                report.removed_blocks += 1;
                report.removed_bytes += b.bytes.len() as u64;
                false
            } else {
                true
            }
        });
        report
    }

    /// Size-based eviction for one unpinned topic: while the topic's stored bytes
    /// exceed `max_bytes`, drop its least-recently-touched block, preferring
    /// non-root blocks (a commit root is the last thing to go).
    pub fn evict_over_budget(&mut self, topic: &TopicId, max_bytes: u64) -> GcReport {
        let mut report = GcReport::default();
        if max_bytes == u64::MAX {
            return report;
        }
        loop {
            let (_, bytes) = self.usage(topic);
            if bytes <= max_bytes {
                return report;
            }
            let victim = self
                .blocks
                .iter()
                .filter(|(_, b)| b.topics.contains(topic))
                .min_by_key(|(id, b)| (b.is_commit_root, b.last_touch, **id))
                .map(|(id, _)| *id);
            let Some(victim) = victim else {
                return report;
            };
            // Count the size BEFORE dropping the attribution, and do not fold in
            // `sweep_unattributed`'s own tally: this report counts blocks evicted
            // from THIS topic, so a block still retained by another topic is
            // still one eviction here and is not double-counted there.
            let size = self
                .blocks
                .get(&victim)
                .map(|b| b.bytes.len() as u64)
                .unwrap_or(0);
            if let Some(b) = self.blocks.get_mut(&victim) {
                b.topics.remove(topic);
            }
            report.removed_blocks += 1;
            report.removed_bytes += size;
            self.sweep_unattributed();
        }
    }
}
