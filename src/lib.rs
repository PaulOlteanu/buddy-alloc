use std::{alloc::Layout, ptr};

const MIN_SIZE: usize = 64;

pub struct BuddyAllocator {
    size: usize,
    min_size: usize,
    max_order: usize,
    memory: *mut u8,
    data: Vec<Data>,
    num_free_at_order: Vec<usize>,
    layout: Layout,
}

impl Drop for BuddyAllocator {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.memory, self.layout);
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Data {
    free: bool,
    split: bool,
    initialized: bool,
}

// TODO: I think this is not that useful right now
// since we put all the bookkeeping in the allocator struct
// This will probably be useful when moving stuff out of there
#[derive(Debug)]
struct Header {
    free: bool,
    order: usize,
}

// Private interface
impl BuddyAllocator {
    fn index_to_order(&self, idx: usize) -> usize {
        let height = (self.data.len().ilog2()) - ((idx + 1).ilog2());

        height as usize
    }

    fn order_to_size(&self, order: usize) -> usize {
        self.min_size * 2usize.pow(order as u32)
    }

    fn index_to_size(&self, idx: usize) -> usize {
        self.order_to_size(self.index_to_order(idx))
    }

    fn index_to_address(&self, idx: usize) -> *mut u8 {
        let order = self.index_to_order(idx);
        let level = self.max_order - order;
        let start: usize = 2usize.pow(level as u32) - 1;
        let amount_before = (idx - start) * self.order_to_size(order);
        println!("amount before {amount_before}");

        unsafe { self.memory.add(amount_before) }
    }

    fn address_to_index(&self, address: *const u8) -> usize {
        todo!()
    }

    fn get_index(&self, size: usize) -> Option<usize> {
        for order in 0..=self.max_order {
            if self.num_free_at_order[order] != 0
                && self.min_size * 2usize.pow(order as u32) >= size + size_of::<Header>()
            {
                let level = self.max_order - order;

                let start: usize = 2usize.pow(level as u32) - 1;
                let end: usize = 2usize.pow(level as u32 + 1) - 2 + 1;

                for idx in start..end {
                    if self.data[idx].free {
                        return Some(idx);
                    }
                }
            }
        }

        None
    }
}

// Public interface
impl BuddyAllocator {
    pub fn new(requested_size: usize) -> Self {
        let size = (requested_size.max(MIN_SIZE)).next_power_of_two();

        tracing::trace!(
            requested_size,
            actual_size = size,
            "creating BuddyAllocater"
        );

        let layout = Layout::from_size_align(size, align_of::<Header>()).unwrap();
        let memory = unsafe { std::alloc::alloc(layout) };

        let order = (size / MIN_SIZE).ilog2();
        tracing::trace!("initial order {}", order);
        let header = Header {
            free: true,
            order: order as usize,
        };

        unsafe { ptr::write(memory.cast(), header) };

        tracing::trace!("initial header at {:p}", memory);

        let max_order = (size / MIN_SIZE).ilog2() as usize;
        let max_nodes = (1 << (max_order + 1)) - 1;

        let mut data = vec![
            Data {
                free: true,
                split: false,
                initialized: false,
            };
            max_nodes
        ];

        data[0].initialized = true;

        let mut free_at_level = vec![0; max_order + 1];
        free_at_level[max_order] = 1;

        Self {
            size,
            min_size: MIN_SIZE,
            max_order,
            memory,
            layout,
            num_free_at_order: free_at_level,
            data,
        }
    }

    /// # Safety
    ///
    /// Pointers allocated *must not* be used after the `BuddyAllocator` is dropped
    pub unsafe fn malloc(&mut self, size: usize, alignment: Option<usize>) -> Option<*mut u8> {
        let required_size = alignment.unwrap_or(size).max(size);

        if required_size >= self.size + size_of::<Header>() {
            return None;
        }

        let mut block_idx = self.get_index(size)?;
        let mut block_order = self.index_to_order(block_idx);

        tracing::trace!(
            "starting from index {} order {} size {}",
            block_idx,
            block_order,
            self.order_to_size(block_order)
        );

        while self.order_to_size(block_order - 1) >= size + size_of::<Header>() {
            let block = &mut self.data[block_idx];
            block.free = false;
            block.split = true;

            block_idx = block_idx * 2 + 1;
            block_order -= 1;

            tracing::trace!(
                "moving down a level to index {} order {} size {}",
                block_idx,
                block_order,
                self.order_to_size(block_order)
            );

            let buddy = block_idx + 1;
            self.num_free_at_order[block_order] += 1;
            self.data[buddy].free = true;

            println!("marking buddy free index {buddy} order {block_order}");
        }

        tracing::trace!(
            "creating block at index {} order {}",
            block_idx,
            self.index_to_order(block_idx)
        );

        let block_data = &mut self.data[block_idx];
        block_data.free = false;
        block_data.initialized = true;

        let header = Header {
            free: false,
            order: block_order,
        };

        let addr = self.index_to_address(block_idx);

        unsafe { ptr::write(addr.cast(), header) }

        Some(unsafe { addr.add(size_of::<Header>()) })
    }

    pub fn free(&mut self, ptr: *mut u8) {
        todo!()
    }
}
