pub struct KeyAllocator {
    // Gaps are keys within the array that have been freed
    // They can be used to construct a new valid unique key
    gaps: Vec<Key>,
    // The next index to allocate a key for
    next: u32,
}

impl KeyAllocator {
    pub fn new() -> KeyAllocator {
        KeyAllocator {
            gaps: Vec::new(),
            next: 0,
        }
    }

    pub fn alloc(&mut self) -> Key {
        match self.gaps.pop() {
            Some(key) => key,
            None => {
                let key = Key {
                    generation: 1,
                    index: self.next,
                };
                self.next += 1;
                key
            }
        }
    }

    pub fn free(&mut self, key: Key) {
        self.gaps.push(Key {
            generation: key.generation + 1,
            index: key.index,
        });
    }
}

#[derive(Copy, Clone)]
pub struct Key {
    generation: u32,
    index: u32,
}

impl Key {
    pub fn null() -> Key {
        Key {
            generation: 0,
            index: 0,
        }
    }
}

pub struct UniqueStore<T: Copy> {
    buffer: Vec<(u32, T)>
}

impl<T: Copy> UniqueStore<T> {
    pub fn new() -> UniqueStore<T> {
        UniqueStore {
            buffer: Vec::new()
        }
    }

    pub fn get(&self, key: Key) -> Option<&T> {
        self.buffer.get(key.index as usize)
            .and_then(|(generation, value)| match key.generation == *generation {
                true => Some(value),
                false => None
            })
    }

    pub fn get_mut(&mut self, key: Key) -> Option<&mut T> {
        self.buffer.get_mut(key.index as usize)
            .and_then(|(generation, value)| match key.generation == *generation {
                true => Some(value),
                false => None
            })
    }

    pub fn contains(&self, key: Key) -> bool {
        match self.get(key) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn insert(&mut self, key: Key, value: T) {
        while self.buffer.len() < key.index as usize {
            self.buffer.push((0, value));
        }
        if self.buffer.len() == key.index as usize {
            self.buffer.push((key.generation, value));
        } else {
            self.buffer[key.index as usize] = (key.generation, value);
        }
    }

    pub fn remove(&mut self, key: Key) {
        self.buffer[key.index as usize].0 = 0;
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.buffer
            .iter()
            .filter_map(|(generation, value)| match generation {
                0 => None,
                _ => Some(value)
            })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.buffer
            .iter_mut()
            .filter_map(|(generation, value)| match generation {
                0 => None,
                _ => Some(value)
            })
    }
}
