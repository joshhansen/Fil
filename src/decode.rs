use cv::Mat;

use rand;
use rand::distributions::{IndependentSample, Range};


use super::util::{MovingAvg,MostFrequent};

const SAMPLES: u64 = 1000;

pub trait Decoder {
    /// FIXME: When const generics happen, use a parameterized array rather than vector
    fn decode(&mut self, image: &Mat) -> Option<&Vec<bool>>;
}

fn sample_color(image: &Mat, x_min: usize, x_max: usize, y_min: usize, y_max: usize) -> [u8; 3] {
    let x_range = Range::new(x_min, x_max);
    let y_range = Range::new(y_min, y_max);

    let mut totals: [u64; 3] = [0,0,0];
    let mut rng = rand::thread_rng();
    for _ in 0..SAMPLES {
        let x = x_range.ind_sample(&mut rng);
        let y = y_range.ind_sample(&mut rng);

        // let (r,g,b):(u8,u8,u8) = image.at2(x as isize, y as isize);
        //FIXME fix this so hard---once cv-rs stops reversing rgb triples
        //FIXME Also, why do we have to reverse the x and y coordinates?
        let (b,g,r):(u8,u8,u8) = image.at2(y as isize, x as isize);

        // eprintln!("{} {} {} {} {},{} -> {},{},{}", x_min, x_max, y_min, y_max, x, y, r, g, b);
        totals[0] += r as u64;
        totals[1] += g as u64;
        totals[2] += b as u64;
    }

    let averages: [u8; 3] = [(totals[0]/SAMPLES) as u8, (totals[1]/SAMPLES) as u8, (totals[2]/SAMPLES) as u8];
    averages
}

#[derive(Clone,Copy,Debug,Eq,Hash,PartialEq)]
enum Desc {
    Clock,
    Signal,
    Both,
    Neither
}
pub struct TimedColorCodedOneBitDecoder {
    bits: Vec<bool>,
    prev_clock: bool,
    left_g_avg: MovingAvg,
    right_r_avg: MovingAvg,
    most_freq_desc: MostFrequent<Desc>
}
impl TimedColorCodedOneBitDecoder {
    pub fn new() -> Self {
        Self {
            bits: vec![false],
            prev_clock: false,
            left_g_avg: MovingAvg::new(1),
            right_r_avg: MovingAvg::new(1),
            most_freq_desc: MostFrequent::new(5)
        }
    }
}
impl Decoder for TimedColorCodedOneBitDecoder {
    fn decode(&mut self, image: &Mat) -> Option<&Vec<bool>> {
        let size = image.size();
        let left_rgb = sample_color(&image, 0, 100, 0, size.height as usize);
        let right_rgb = sample_color(&image, (size.width - 100) as usize, size.width as usize, 0, size.height as usize);

        let g_avg = self.left_g_avg.push(left_rgb[1] as f64);
        let r_avg = self.right_r_avg.push(right_rgb[0] as f64);

        let desc = if r_avg < 20f64 && g_avg < 20f64 {
            Desc::Neither
        } else if r_avg > 60f64 && g_avg > 60f64 {
            Desc::Both
        } else if r_avg > g_avg {
            Desc::Clock
        } else {
            Desc::Signal
        };

        let desc = self.most_freq_desc.push(desc);
        // eprintln!("{:?} {:?} {:?} {:?} {:?} {:?}", desc, rgb, top_rgb, bottom_rgb, left_rgb, right_rgb);

        let (signal, clock) = match desc {
            Desc::Signal => (true, false),
            Desc::Clock => (false, true),
            Desc::Both => (true, true),
            Desc::Neither => (false, false)
        };

        let result = if clock != self.prev_clock {
            eprintln!("{} {}", signal, clock);

            self.bits[0] = signal;
            Some(&self.bits)
        } else {
            None
        };

        self.prev_clock = clock;

        result
    }
}
