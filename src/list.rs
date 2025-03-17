pub struct List<T> {
    head: Option<Link<T>>,
    tail: Option<Link<T>>,
}

struct Link<T> {
    val: T,
}

impl<T> List<T> {
    pub const fn new() -> Self {
        todo!()
    }
}
