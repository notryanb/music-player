
pub struct Scope {
    pub write_idx: usize,
    pub buffer: Vec<f32>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            write_idx: 0,
            buffer: vec![0.0f32; 48000 * 3],
        }
    }

    pub fn write_sample(&mut self, sample: f32) {
        if self.write_idx >= self.buffer.len() {
            self.write_idx -= self.buffer.len();
        }

        self.buffer[self.write_idx] = sample;
        self.write_idx += 1;
    }
}

impl<'a> IntoIterator for &'a Scope {
    type Item = f32;
    type IntoIter = ScopeIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let mut index = self.write_idx + 1;

        if &index >= &self.buffer.len() {
            index -= &self.buffer.len();
        }

        ScopeIterator {
            scope: self,
            index,
        }
    }
}
pub struct ScopeIterator<'a> {
    scope: &'a Scope,
    index: usize,
}

impl<'a> Iterator for ScopeIterator<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let buffer = &self.scope.buffer;
        self.index += 1;

        if self.index >= buffer.len() {
            self.index -= buffer.len();
        }

        if self.index == self.scope.write_idx {
            return None;
        }

        Some(buffer[self.index]) 
    }
}
