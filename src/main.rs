use buddy_alloc::BuddyAllocator;
use tracing::Level;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    // let mut allocator = BuddyAllocator::new(1 << 10);
    let mut allocator = BuddyAllocator::new(1 << 10);

    let p = unsafe { allocator.malloc(64, None) };
    println!("{:?}", p);
    let p = unsafe { allocator.malloc(64, None) };
    println!("{:?}", p);
    let p = unsafe { allocator.malloc(64, None) };
    println!("{:?}", p);
    let p = unsafe { allocator.malloc(64, None) };
    println!("{:?}", p);
}
