//! Garbage Collector: Memory management for managed mode
//!
//! The GC is responsible for:
//! - Tracing live objects from roots
//! - Marking dead objects
//! - Collecting unreachable memory
//! - Maintaining heap invariants

use std::collections::HashMap as StdHashMap;

/// A reference to a heap object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectRef(u32);

/// Metadata for a heap object.
#[derive(Debug, Clone)]
pub struct ObjectMetadata {
    pub size: usize,
    pub marked: bool,
    pub refcount: u32, // Reference count for cycle detection
}

/// The garbage collector.
pub struct GarbageCollector {
    heap: StdHashMap<ObjectRef, (Vec<u8>, ObjectMetadata)>,
    object_counter: u32,
    roots: Vec<ObjectRef>,
}

impl GarbageCollector {
    pub fn new() -> Self {
        GarbageCollector {
            heap: Default::default(),
            object_counter: 0,
            roots: Vec::new(),
        }
    }

    /// Allocate a new object on the heap.
    pub fn allocate(&mut self, data: Vec<u8>) -> ObjectRef {
        let obj_ref = ObjectRef(self.object_counter);
        self.object_counter += 1;

        let metadata = ObjectMetadata {
            size: data.len(),
            marked: false,
            refcount: 0,
        };

        self.heap.insert(obj_ref, (data, metadata));
        obj_ref
    }

    /// Add a root reference (reachable from the stack).
    pub fn add_root(&mut self, obj_ref: ObjectRef) {
        if !self.roots.contains(&obj_ref) {
            self.roots.push(obj_ref);
        }
    }

    /// Remove a root reference.
    pub fn remove_root(&mut self, obj_ref: ObjectRef) {
        self.roots.retain(|&r| r != obj_ref);
    }

    /// Mark phase: trace all reachable objects.
    pub fn mark_phase(&mut self) {
        // Reset all marks
        for (_, (_, metadata)) in self.heap.iter_mut() {
            metadata.marked = false;
        }

        // Mark all reachable objects starting from roots
        let roots = self.roots.clone();
        for &root in &roots {
            self.mark_recursive(root);
        }
    }

    fn mark_recursive(&mut self, obj_ref: ObjectRef) {
        if let Some((_, metadata)) = self.heap.get_mut(&obj_ref) {
            if !metadata.marked {
                metadata.marked = true;
                // TODO: Trace object references and recursively mark
            }
        }
    }

    /// Sweep phase: collect unmarked (dead) objects.
    pub fn sweep_phase(&mut self) -> usize {
        let mut collected = 0;

        self.heap.retain(|_, (_, metadata)| {
            if !metadata.marked {
                collected += 1;
                false
            } else {
                true
            }
        });

        collected
    }

    /// Full collection cycle: mark + sweep.
    pub fn collect(&mut self) -> GCStats {
        let before = self.heap.len();

        self.mark_phase();
        let collected = self.sweep_phase();

        let after = self.heap.len();

        GCStats {
            before,
            after,
            collected,
        }
    }

    /// Current heap size in bytes.
    pub fn heap_size(&self) -> usize {
        self.heap.values().map(|(data, _)| data.len()).sum()
    }

    /// Number of live objects.
    pub fn live_objects(&self) -> usize {
        self.heap.len()
    }
}

/// Statistics from a garbage collection run.
#[derive(Debug)]
pub struct GCStats {
    pub before: usize,
    pub after: usize,
    pub collected: usize,
}

// TODO: Implement generational GC for efficiency
// TODO: Implement incremental GC (stop-the-world)
// TODO: Implement cycle detection (reference counting + GC)
// TODO: Implement heap compaction
// TODO: Implement memory pressure thresholds

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate() {
        let mut gc = GarbageCollector::new();
        let obj = gc.allocate(vec![1, 2, 3]);
        assert_eq!(gc.live_objects(), 1);
    }

    #[test]
    fn test_mark_and_sweep() {
        let mut gc = GarbageCollector::new();
        let obj1 = gc.allocate(vec![1, 2, 3]);
        let _obj2 = gc.allocate(vec![4, 5, 6]); // Not in roots

        gc.add_root(obj1);
        let stats = gc.collect();

        assert_eq!(stats.before, 2);
        assert_eq!(stats.after, 1);
        assert_eq!(stats.collected, 1);
    }
}
