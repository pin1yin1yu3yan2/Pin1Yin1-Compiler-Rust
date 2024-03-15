#[derive(Debug, Clone)]
pub struct Source<S = char> {
    file_name: String,
    inner: Vec<S>,
}

impl<S> Source<S> {
    pub fn new(file_name: impl Into<String>, iter: impl Iterator<Item = S>) -> Self {
        Self {
            file_name: file_name.into(),
            inner: iter.collect(),
        }
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }
}

impl<S> std::ops::Deref for Source<S> {
    type Target = [S];

    fn deref(&self) -> &Self::Target {
        &self.inner[..]
    }
}
