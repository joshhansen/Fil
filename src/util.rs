use std::collections::HashMap;
use std::hash::Hash;

pub struct MostFrequent<T> {
    items: Vec<T>,
    length: usize,
    counts: HashMap<T,u64>
}
impl <T:Copy+Eq+Hash> MostFrequent<T> {
    pub fn new(length: usize) -> Self {
        Self {
            items: Vec::new(),
            length: length,
            counts: HashMap::new()
        }
    }

    pub fn push(&mut self, item: T) -> T {
        self.items.push(item);
        if self.items.len() > self.length {
            self.items.remove(0);
        }

        self.counts.clear();
        for item in self.items.iter() {
            let new_count = self.counts.get(item).map_or(1, |count| *count+1);
            self.counts.insert(*item, new_count);
        }
        let mut most_frequent: Option<T> = None;
        let mut highest_frequency: Option<u64> = None;
        for (k,v) in self.counts.iter() {
            if most_frequent.is_none() || *v > highest_frequency.unwrap() {
                most_frequent = Some(*k);
                highest_frequency = Some(*v);
            }
        }
        most_frequent.unwrap()
    }
}

pub struct MovingAvg {
    length: usize,
    values: Vec<f64>
}
impl MovingAvg {
    pub fn new(length: usize) -> Self {
        Self {
            length: length,
            values: Vec::with_capacity(length + 1)
        }
    }

    pub fn avg(&self) -> f64 {
        let mut sum: f64 = 0.0;
        for value in self.values.iter() {
            sum += *value;
        }

        sum / self.values.len() as f64
    }

    pub fn push(&mut self, value: f64) -> f64 {
        self.values.push(value);
        if self.values.len() > self.length {
            self.values.remove(0);
        }
        self.avg()
    }
}
