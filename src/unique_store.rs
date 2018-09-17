use std::{
    ops::{Index, IndexMut},
    iter::Peekable
};

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

#[derive(Copy, Clone, Eq, PartialEq)]
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

    pub fn iter(&self) -> impl Iterator<Item = (Key, &T)> {
        convert_iter(self.buffer
            .iter()
            .map(|(generation, value)| (*generation, value))
            .enumerate())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Key, &mut T)> {
        convert_iter(self.buffer
            .iter_mut()
            .map(|(generation, value)| (*generation, value))
            .enumerate())
    }
}

impl<T: Copy> Index<Key> for UniqueStore<T> {
    type Output = T;

    fn index(&self, key: Key) -> &T {
        self.get(key).unwrap()
    }
}

impl<T: Copy> IndexMut<Key> for UniqueStore<T> {
    fn index_mut(&mut self, key: Key) -> &mut T {
        self.get_mut(key).unwrap()
    }
}

fn convert_iter<T>(x: impl Iterator<Item = (usize, (u32, T))>) -> impl Iterator<Item = (Key, T)> {
    x.filter_map(|(index, (generation, value))| match generation {
        0 => None,
        _ => Some((Key {
            generation,
            index: index as u32
        }, value))
    })
}

pub fn join_key<T, U>(a: impl Iterator<Item = (Key, T)>, b: impl Iterator<Item = (Key, U)>) -> impl Iterator<Item = (Key, (T, U))> {
    Joined {
        a: a.peekable(),
        b: b.peekable()
    }
}

struct Joined<T, U, A: Iterator<Item = (Key, T)>, B: Iterator<Item = (Key, U)>> {
    a: Peekable<A>,
    b: Peekable<B>,
}

impl<T, U, A: Iterator<Item = (Key, T)>, B: Iterator<Item = (Key, U)>> Iterator for Joined<T, U, A, B> {
    type Item = (Key, (T, U));

    fn next(&mut self) -> Option<Self::Item> {
        while self.a.peek().is_some() && self.b.peek().is_some() {
            let a_key = self.a.peek().unwrap().0;
            let b_key = self.b.peek().unwrap().0;
            if a_key == b_key {
                let (_, a) = self.a.next().unwrap();
                let (key, b) = self.b.next().unwrap();
                return Some((key, (a, b)));
            } else if a_key.index < b_key.index {
                self.a.next();
            } else {
                self.b.next();
            }
        }
        None
    }
}
