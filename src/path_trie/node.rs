pub struct Node<T> {
    path: Path,
    data: Option<T>,
    children: Vec<Self>,
}

enum Path {
    Static(String),
    Param(String),
    Wildcard,
}

impl<T> Node<T> {
    #[inline]
    pub fn new(path: Path) -> Self {
        Self {
            path,
            data: None,
            children: Vec::new(),
        }
    }

    pub fn insert(&mut self) {
    }

    pub fn get(&self, path: &str) -> Option<T> {
        None
    }
}