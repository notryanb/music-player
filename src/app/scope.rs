
pub struct Scope {
    pub write_idx: usize,
    pub buffer: Vec<f32>,
}

impl Scope {
    // TODO - take in ms and figure out buffer size
    pub fn new() -> Self {
        Self {
            write_idx: 0,
            buffer: vec![0.0f32; 48000 * 1],
        }
    }

    pub fn write_sample(&mut self, sample: f32) {
        if self.write_idx >= self.buffer.len() {
            self.write_idx -= self.buffer.len();
        }

        self.buffer[self.write_idx] = sample;
        self.write_idx += 1;
    }

    pub fn write_samples(&mut self, samples: &[f32]) {
        // if slice can fit from write pointer to end, then write all at once.
        // Otherwise, write from write idx to end of buf.
        // then update write idx to beginning and write the remaining of slice.
        if samples.len() > 0 && samples.len() <= self.buffer.len() - self.write_idx {
            self.buffer[self.write_idx..(samples.len() + self.write_idx)].copy_from_slice(&samples);
            self.write_idx += samples.len();
        } else {
            let remaining = self.buffer.len() - self.write_idx;
            self.buffer[self.write_idx..].copy_from_slice(&samples[..remaining]);
            self.write_idx= 0;
            self.buffer[self.write_idx..(samples.len() - remaining)].copy_from_slice(&samples[remaining..]);
        }
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
            counter: 0,
        }
    }
}
pub struct ScopeIterator<'a> {
    scope: &'a Scope,
    index: usize,
    counter: usize,
}

impl<'a> Iterator for ScopeIterator<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.counter += 1;

        if self.index >= self.scope.buffer.len() {
            self.index -= self.scope.buffer.len();
        }

        // I bet this is the cause of the issue. Entering an infinite loop
        if self.index == self.scope.write_idx || self.counter > self.scope.buffer.len() {
            return None;
        }

        Some(self.scope.buffer[self.index]) 
    }
}

