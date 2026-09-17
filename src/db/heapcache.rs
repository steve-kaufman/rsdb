use std::fmt::Debug;
use std::{collections::HashMap, hash::Hash};

use crate::db::{HEAP_CACHE_SIZE, PAGE_SIZE, PageId};

const NUM_PAGES: usize = HEAP_CACHE_SIZE / PAGE_SIZE;

#[derive(Debug)]
struct CacheEntry<K, V> {
    key: K,
    value: V,
    next: Option<usize>,
    prev: Option<usize>,
}

pub struct LRUCache<K, V, const SIZE: usize> {
    entries: Box<[Option<CacheEntry<K, V>>; SIZE]>,
    lookup: HashMap<K, usize>,
    head: usize,
    tail: usize,
}

impl<K, V, const SIZE: usize> LRUCache<K, V, SIZE>
where
    K: Eq + Hash + Clone + Debug,
    V: Debug,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, k: K, v: V) {
        let index = self.allocate_new_index();
        // Write cache entry at index and automatically set as new head
        let mut entry = CacheEntry {
            key: k.clone(),
            value: v,
            next: None,
            prev: None,
        };
        if let Some(head) = self.get_entry_mut(self.head) {
            head.next = Some(index);
            entry.prev = Some(self.head);
        }
        self.entries[index] = Some(entry);
        self.lookup.insert(k, index);
        self.head = index;
    }

    pub fn read(&mut self, k: &K) -> Option<&V> {
        let index = *self.lookup.get(k)?;
        self.touch(index);
        Some(&self.get_entry(index).unwrap().value)
    }

    fn allocate_new_index(&mut self) -> usize {
        if self.lookup.len() < SIZE {
            // Use next available index if cache not full yet
            self.lookup.len()
        } else {
            // Overwrite and evict current tail if cache is full
            let old_tail_index = self.tail;
            let old_tail_key = self.get_entry(old_tail_index).unwrap().key.clone();

            let new_tail_index = self.get_entry(self.tail).unwrap().next.unwrap();
            self.get_entry_mut(new_tail_index).unwrap().prev = None;
            self.tail = new_tail_index;

            self.lookup.remove(&old_tail_key);
            old_tail_index
        }
    }

    fn touch(&mut self, index: usize) {
        if let Some(next) = self.get_entry(index).unwrap().next {
            self.move_to_head(index, next);
        }
    }

    fn move_to_head(&mut self, index: usize, next: usize) {
        let prev = self.get_entry(index).unwrap().prev;

        // Link next with previous (or none if this was tail)
        self.get_entry_mut(next).unwrap().prev = prev;

        // If this was not the tail, link previous with next
        // Else, make next the new tail
        if let Some(prev) = prev {
            self.get_entry_mut(prev).unwrap().next = Some(next);
        } else {
            self.tail = next;
        }

        // Make this the new head
        let old_head = self.head;
        let entry = self.get_entry_mut(index).unwrap();
        entry.prev = Some(old_head);
        entry.next = None;
        self.get_entry_mut(old_head).unwrap().next = Some(index);
        self.head = index;
    }

    fn get_entry(&self, index: usize) -> Option<&CacheEntry<K, V>> {
        self.entries[index].as_ref()
    }

    fn get_entry_mut(&mut self, index: usize) -> Option<&mut CacheEntry<K, V>> {
        self.entries[index].as_mut()
    }
}

impl<K, V, const SIZE: usize> Default for LRUCache<K, V, SIZE> {
    fn default() -> Self {
        Self {
            entries: Box::new(std::array::from_fn(|_| None)),
            lookup: HashMap::new(),
            head: 0,
            tail: 0,
        }
    }
}

pub type Page = [u8; PAGE_SIZE];

pub type HeapCache = LRUCache<PageId, Page, NUM_PAGES>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru() {
        let mut lru = LRUCache::<String, String, 3>::new();

        struct Page {
            name: String,
            contents: String,
        }

        let pages = [
            Page {
                name: "page0".to_string(),
                contents: "page0 contents".to_string(),
            },
            Page {
                name: "page1".to_string(),
                contents: "page1 contents".to_string(),
            },
            Page {
                name: "page2".to_string(),
                contents: "page2 contents".to_string(),
            },
            Page {
                name: "page3".to_string(),
                contents: "page3 contents".to_string(),
            },
            Page {
                name: "page4".to_string(),
                contents: "page4 contents".to_string(),
            },
        ];

        // First page
        assert_eq!(lru.read(&pages[0].name), None);

        lru.insert(pages[0].name.clone(), pages[0].contents.clone());

        assert_eq!(
            lru.read(&pages[0].name).cloned(),
            Some(pages[0].contents.clone())
        );

        // Second page
        assert_eq!(lru.read(&pages[1].name), None);

        lru.insert(pages[1].name.clone(), pages[1].contents.clone());

        assert_eq!(
            lru.read(&pages[1].name).cloned(),
            Some(pages[1].contents.clone())
        );
        assert_eq!(
            lru.read(&pages[0].name).cloned(),
            Some(pages[0].contents.clone())
        );

        // Third page
        assert_eq!(lru.read(&pages[2].name), None);

        lru.insert(pages[2].name.clone(), pages[2].contents.clone());

        assert_eq!(
            lru.read(&pages[2].name).cloned(),
            Some(pages[2].contents.clone())
        );
        assert_eq!(
            lru.read(&pages[1].name).cloned(),
            Some(pages[1].contents.clone())
        );
        assert_eq!(
            lru.read(&pages[0].name).cloned(),
            Some(pages[0].contents.clone())
        );

        // Fourth page
        assert_eq!(lru.read(&pages[3].name), None);

        lru.insert(pages[3].name.clone(), pages[3].contents.clone());

        assert_eq!(
            lru.read(&pages[3].name).cloned(),
            Some(pages[3].contents.clone())
        );
        assert_eq!(lru.read(&pages[2].name).cloned(), None);
        assert_eq!(
            lru.read(&pages[1].name).cloned(),
            Some(pages[1].contents.clone())
        );
        assert_eq!(
            lru.read(&pages[0].name).cloned(),
            Some(pages[0].contents.clone())
        );
    }

    #[test]
    fn test_lru_insert() {
        let mut lru = LRUCache::<String, String, 3>::new();

        struct Page {
            name: String,
            contents: String,
        }

        let pages = [
            Page {
                name: "page1".to_string(),
                contents: "page1 contents".to_string(),
            },
            Page {
                name: "page2".to_string(),
                contents: "page2 contents".to_string(),
            },
            Page {
                name: "page3".to_string(),
                contents: "page3 contents".to_string(),
            },
            Page {
                name: "page4".to_string(),
                contents: "page4 contents".to_string(),
            },
            Page {
                name: "page5".to_string(),
                contents: "page5 contents".to_string(),
            },
        ];

        for page in &pages {
            lru.insert(page.name.clone(), page.contents.clone());
        }
    }
}
