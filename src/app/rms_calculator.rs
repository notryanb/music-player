use std::collections::VecDeque;

// A running sum windowed RMS calculator.
#[derive(Default)]
pub struct RmsCalculator {
    squared_samples: VecDeque<f32>,
    sum: f32,
    capacity: usize,
    index: usize,
}

impl RmsCalculator {
    pub fn new(size: usize) -> Self {
        Self {
            // I think I need 1 to 2 more samples than the sample window because I don't want to hit the limit
            // and fail to push back or resize the buffer
            squared_samples: VecDeque::with_capacity(size + 2),
            sum: 0.0,
            capacity: size,
            index: 0,
        }
    }

    pub fn reset(&mut self) {
        self.squared_samples.clear();
        self.sum = 0.0;
        self.index = 0;
    }

    pub fn set_window_size(&mut self, capacity: usize) {
        self.capacity = capacity;
    }

    pub fn write_sample(&mut self, sample: f32) {
        while self.index >= self.capacity {
            if let Some(oldest_squared_sample) = self.squared_samples.pop_front() {
                self.sum -= oldest_squared_sample;
                self.index -= 1;
            }
        }

        self.squared_samples.push_back(sample * sample);
        self.sum += sample * sample;
        self.index += 1;
    }

    pub fn get_rms_value(&self) -> f32 {
        (self.sum / self.capacity as f32).sqrt()
    }
}
