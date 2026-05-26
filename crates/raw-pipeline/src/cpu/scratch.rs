use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

const POOL_CAP: usize = 8;

thread_local! {
    static POOL: RefCell<Vec<Vec<f32>>> = const { RefCell::new(Vec::new()) };
}

pub struct Scratch(Option<Vec<f32>>);

impl std::fmt::Debug for Scratch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scratch")
            .field("len", &self.0.as_ref().map(|v| v.len()).unwrap_or(0))
            .finish()
    }
}

impl Scratch {
    pub fn take_zeroed(len: usize) -> Self {
        let mut v = checkout(len);
        v.clear();
        v.resize(len, 0.0);
        Self(Some(v))
    }

    pub fn take_uninit(len: usize) -> Self {
        let mut v = checkout(len);
        v.clear();
        v.reserve(len);
        #[allow(clippy::uninit_vec)]
        unsafe {
            v.set_len(len);
        }
        Self(Some(v))
    }

    pub fn as_slice(&self) -> &[f32] {
        self.0.as_ref().unwrap()
    }

    pub fn as_mut_slice(&mut self) -> &mut [f32] {
        self.0.as_mut().unwrap()
    }
}

impl Deref for Scratch {
    type Target = [f32];
    fn deref(&self) -> &[f32] {
        self.as_slice()
    }
}

impl DerefMut for Scratch {
    fn deref_mut(&mut self) -> &mut [f32] {
        self.as_mut_slice()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let Some(v) = self.0.take() else {
            return;
        };
        POOL.with(|p| {
            let mut pool = p.borrow_mut();
            if pool.len() < POOL_CAP {
                pool.push(v);
            }
        });
    }
}

fn checkout(len: usize) -> Vec<f32> {
    POOL.with(|p| {
        let mut pool = p.borrow_mut();
        if pool.is_empty() {
            return Vec::with_capacity(len);
        }
        let mut best_idx = 0;
        let mut best_cap = pool[0].capacity();
        for (i, b) in pool.iter().enumerate().skip(1) {
            let c = b.capacity();
            if (best_cap < len && c > best_cap) || (c >= len && (best_cap < len || c < best_cap)) {
                best_idx = i;
                best_cap = c;
            }
        }
        pool.swap_remove(best_idx)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reuse_preserves_capacity() {
        let cap_after_drop;
        {
            let s = Scratch::take_zeroed(1000);
            cap_after_drop = s.0.as_ref().unwrap().capacity();
        }
        let s2 = Scratch::take_zeroed(500);
        if s2.0.as_ref().unwrap().capacity() < cap_after_drop {
            panic!("scratch did not reuse larger buffer");
        }
    }

    #[test]
    fn zeroed_initializes_to_zero() {
        let s = Scratch::take_zeroed(64);
        for &v in s.iter() {
            if v != 0.0 {
                panic!("non-zero in zeroed scratch");
            }
        }
    }

    #[test]
    fn uninit_has_correct_len() {
        let s = Scratch::take_uninit(100);
        if s.len() != 100 {
            panic!("uninit len mismatch");
        }
    }
}
