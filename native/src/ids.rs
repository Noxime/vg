use vg_types::PlayerId;

pub trait Id {
    const FIRST: Self;

    /// Increment self and return the value
    fn incr(&mut self) -> Self;
}

// This source generates new unique ID's, while re-assigning the free'd IDs
pub struct IdSource<T> {
    head: T,
    freed: Vec<T>,
}

impl<T: Id> IdSource<T> {
    pub fn new() -> IdSource<T> {
        IdSource {
            head: Id::FIRST,
            freed: vec![],
        }
    }

    pub fn alloc(&mut self) -> T {
        self.freed.pop().unwrap_or_else(|| self.head.incr())
    }

    pub fn free(&mut self, id: T) {
        self.freed.push(id)
    }
}

impl Id for PlayerId {
    const FIRST: Self = PlayerId(0);

    fn incr(&mut self) -> Self {
        self.0 += 1;
        PlayerId(self.0)
    }
}
