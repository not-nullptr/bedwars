pub struct EntityAllocator {
    next_id: i32,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self { next_id: 1 }
    }

    pub fn allocate(&mut self) -> i32 {
        let id = self.next_id;
        self.next_id = if self.next_id == i32::MAX {
            1
        } else {
            self.next_id + 1
        };

        id
    }
}
