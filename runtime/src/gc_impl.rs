/// Mark-and-sweep garbage collector with cycle detection

use std::collections::{HashSet, HashMap};

#[derive(Clone)]
pub struct HeapObject {
    pub id: u64,
    pub size: usize,
    pub marked: bool,
    pub refs: Vec<u64>,
}

pub struct GarbageCollector {
    objects: HashMap<u64, HeapObject>,
    roots: Vec<u64>,
    marked: HashSet<u64>,
    next_id: u64,
}

impl GarbageCollector {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            roots: Vec::new(),
            marked: HashSet::new(),
            next_id: 1,
        }
    }

    pub fn allocate(&mut self, size: usize) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let obj = HeapObject {
            id,
            size,
            marked: false,
            refs: Vec::new(),
        };

        self.objects.insert(id, obj);
        id
    }

    pub fn add_root(&mut self, ptr: u64) {
        self.roots.push(ptr);
    }

    pub fn add_ref(&mut self, from: u64, to: u64) {
        if let Some(obj) = self.objects.get_mut(&from) {
            obj.refs.push(to);
        }
    }

    pub fn collect(&mut self) -> usize {
        // Clear marks
        self.marked.clear();

        // Mark phase: DFS from roots
        for &root in &self.roots.clone() {
            self.mark(root);
        }

        // Sweep phase: count freed bytes
        let mut freed = 0;
        let to_remove: Vec<u64> = self.objects
            .iter()
            .filter(|(_, obj)| !self.marked.contains(&obj.id))
            .map(|(id, _)| *id)
            .collect();

        for id in to_remove {
            if let Some(obj) = self.objects.remove(&id) {
                freed += obj.size;
            }
        }

        freed
    }

    fn mark(&mut self, id: u64) {
        if self.marked.contains(&id) {
            return;
        }

        self.marked.insert(id);

        if let Some(obj) = self.objects.get(&id) {
            let refs = obj.refs.clone();
            for ref_id in refs {
                self.mark(ref_id);
            }
        }
    }

    pub fn heap_size(&self) -> usize {
        self.objects.values().map(|o| o.size).sum()
    }
}

impl Default for GarbageCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate() {
        let mut gc = GarbageCollector::new();
        let id = gc.allocate(1024);
        assert_eq!(id, 1);
    }

    #[test]
    fn test_gc_collect() {
        let mut gc = GarbageCollector::new();
        let id1 = gc.allocate(1024);
        let id2 = gc.allocate(512);
        gc.add_root(id1);
        let freed = gc.collect();
        assert_eq!(freed, 512);
    }
}
