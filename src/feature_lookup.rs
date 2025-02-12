use std::sync::atomic::{AtomicU8, AtomicU16, Ordering};

pub struct FeatureLookup {
    // What's 66048 bytes of heap space between friends?
    by_id: Box<[AtomicU8; 0xFFFF]>,
    by_index: Box<[AtomicU16; 0xFF]>,
}

impl FeatureLookup {
    pub fn new() -> Self {
        Self {
            by_id: bytemuck::allocation::zeroed_box(),
            by_index: bytemuck::allocation::zeroed_box(),
        }
    }

    pub fn insert(&self, index: u8, id: u16) {
        self.by_index[index as usize].store(id, Ordering::Relaxed);
        self.by_id[id as usize].store(index, Ordering::Relaxed);
    }

    pub fn get_index(&self, id: u16) -> Option<u8> {
        if id == 0 {
            return Some(0);
        }
        let index = self.by_id[id as usize].load(Ordering::Relaxed);
        (index != 0).then_some(index)
    }

    pub fn get_id(&self, index: u8) -> Option<u16> {
        if index == 0 {
            return Some(0);
        }
        let id = self.by_index[index as usize].load(Ordering::Relaxed);
        (id != 0).then_some(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (u8, u16)> + '_ {
        (0..=0xFF).filter_map(|index| {
            let id = self.get_id(index)?;
            Some((index, id))
        })
    }
}

impl Default for FeatureLookup {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for FeatureLookup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();
        for (index, id) in self.iter() {
            map.entry(&index, &id);
        }
        map.finish()
    }
}
