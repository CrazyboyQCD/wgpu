use bumpalo::Bump;

#[derive(Default)]
pub struct Allocator {
    bump: Bump,
}

impl Allocator {
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bump: Bump::with_capacity(capacity),
        }
    }

    pub fn alloc<T>(&self, val: T) -> &mut T {
        const {
            assert!(
                !std::mem::needs_drop::<T>(),
                "Cannot allocate Drop type in arena"
            )
        };

        self.bump.alloc(val)
    }

    pub fn alloc_str<'alloc>(&'alloc self, src: &str) -> &'alloc mut str {
        self.bump.alloc_str(src)
    }

    pub fn reset(&mut self) {
        self.bump.reset();
    }

    pub fn capacity(&self) -> usize {
        self.bump.allocated_bytes()
    }

    pub fn used_bytes(&self) -> usize {
        let mut bytes = 0;
        // SAFETY: No allocations are made while `chunks_iter` is alive. No data is read from the chunks.
        let chunks_iter = unsafe { self.bump.iter_allocated_chunks_raw() };
        for (_, size) in chunks_iter {
            bytes += size;
        }
        bytes
    }

    pub fn bump(&self) -> &Bump {
        &self.bump
    }
}
