
pub struct Scope {
    pub sample_idx: usize,
    pub display_idx: isize,
    pub buffer: Vec<f32>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            sample_idx: 0,
            display_idx: 0,
            buffer: vec![0.0f32; 48000 * 3],
        }
    }

    pub fn write_sample(&mut self, sample: f32) {
        if self.sample_idx >= self.buffer.len() {
            self.sample_idx -= self.buffer.len();
        }

        self.buffer[self.sample_idx] = sample;
        self.sample_idx += 1;
    }
}

// impl Iterator for Scope {
//     type Item = f32;
    
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.display_idx >= self.buffer.len() as isize {
//             return None;
//         }

//         self.display_idx += 1;
//         Some(self.buffer[(self.display_idx - 1) as usize])
//     }    
// }
