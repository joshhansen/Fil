use cv::Mat;

use rand;
use rand::distributions::{IndependentSample, Range};


use super::util::{DiscreteFirstDerivative,MovingAvg,MostFrequent};

const SAMPLES: u64 = 1000;
const BIT_THRESHOLD: u8 = 128;

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
        let (b,g,r):(u8,u8,u8) = image.at2(y as isize, x as isize);//FIXME fix this so hard---once cv-rs stops reversing rgb triples

        // eprintln!("{} {} {} {} {},{} -> {},{},{}", x_min, x_max, y_min, y_max, x, y, r, g, b);
        // totals[0] += scalar.v0 as u64;
        // totals[1] += scalar.v1 as u64;
        // totals[2] += scalar.v2 as u64;
        totals[0] += r as u64;
        totals[1] += g as u64;
        totals[2] += b as u64;
    }

    let averages: [u8; 3] = [(totals[0]/SAMPLES) as u8, (totals[1]/SAMPLES) as u8, (totals[2]/SAMPLES) as u8];
    averages
}

fn avg_color(color: &[u8; 3]) -> u8{
    ((color[0] as u16 + color[1] as u16 + color[2] as u16) / 3) as u8
}

struct GreedyOneBitDecoder {
    bits: Vec<bool>
}

impl GreedyOneBitDecoder {
    fn new() -> Self {
        Self {
            bits: vec![false]
        }
    }
}

impl Decoder for GreedyOneBitDecoder {

    fn decode(&mut self, image: &Mat) -> Option<&Vec<bool>> {
        let size = image.size();
        let color = sample_color(image, 0, size.width as usize, 0, size.height as usize);
        let avg_color = ((color[0] as u16 + color[1] as u16 + color[2] as u16) / 3) as u8;

        self.bits[0] = avg_color > BIT_THRESHOLD;

        Some(&self.bits)
    }
}

#[derive(Clone,Copy,Debug,Eq,Hash,PartialEq)]
enum Desc {
    Right,
    Left,
    Both,
    Neither
}

struct TimedOneBitDecoder {
    bits: Vec<bool>,
    prev_clock: bool,
    signal_color_moving_avg: MovingAvg,
    clock_color_moving_avg: MovingAvg,
    signal_deriv: DiscreteFirstDerivative,
    clock_deriv: DiscreteFirstDerivative,
    signal_color_moving_avg_deriv: DiscreteFirstDerivative,
    clock_color_moving_avg_deriv: DiscreteFirstDerivative,
    most_freq_desc: MostFrequent<Desc>
}

impl TimedOneBitDecoder {
    fn new() -> Self {
        Self {
            bits: vec![false],
            prev_clock: false,
            signal_color_moving_avg: MovingAvg::new(3),
            clock_color_moving_avg: MovingAvg::new(3),
            signal_deriv: DiscreteFirstDerivative::new(),
            clock_deriv: DiscreteFirstDerivative::new(),
            signal_color_moving_avg_deriv: DiscreteFirstDerivative::new(),
            clock_color_moving_avg_deriv: DiscreteFirstDerivative::new(),
            most_freq_desc: MostFrequent::new(3)
        }
    }
}

impl Decoder for TimedOneBitDecoder {
    fn decode(&mut self, image: &Mat) -> Option<&Vec<bool>> {
        let size = image.size();

        let clock_min_x = size.width / 2;

        // let signal_color_rgb = sample_color(image, 0, clock_min_x as usize - 20, 0, size.height as usize);
        // let clock_color_rgb = sample_color(image, clock_min_x as usize + 20, size.width as usize, 0, size.height as usize);

        let top_color = avg_color( &sample_color(image, 0, size.width as usize, 0, size.height as usize / 2 ) );
        let bottom_color = avg_color( &sample_color(image, 0, size.width as usize, size.height as usize / 2, size.height as usize) );

        // let clock_color = sample_color(image, 0, clock_min_x as usize, 0, size.height as usize);
        // let signal_color = sample_color(image, clock_min_x as usize, size.width as usize, 0, size.height as usize);

        // let signal_color = avg_color(&signal_color_rgb) as f64;
        // let clock_color = avg_color(&clock_color_rgb) as f64;

        let signal_color = top_color as f64;
        let clock_color = bottom_color as f64;

        let delta = signal_color - clock_color;

        let signal_deriv = self.signal_deriv.push(signal_color);
        let clock_deriv = self.clock_deriv.push(clock_color);

        let signal_color_moving_avg = self.signal_color_moving_avg.push( signal_color) as u8;
        let clock_color_moving_avg = self.clock_color_moving_avg.push( clock_color) as u8;
        let moving_avg_delta = (signal_color_moving_avg as i16 - clock_color_moving_avg as i16) as i8;

        let signal_color_moving_avg_deriv = self.signal_color_moving_avg_deriv.push(signal_color_moving_avg as f64);
        let clock_color_moving_avg_deriv = self.clock_color_moving_avg_deriv.push(clock_color_moving_avg as f64);

        // if signal_deriv.is_some() {
        //     eprintln!("{} {} {} {} {} {} {} {} {} {}",
        //         signal_color,
        //         clock_color,
        //         delta,
        //         signal_deriv.unwrap(),
        //         clock_deriv.unwrap(),
        //         signal_color_moving_avg,
        //         clock_color_moving_avg,
        //         moving_avg_delta,
        //         signal_color_moving_avg_deriv.unwrap(),
        //         clock_color_moving_avg_deriv.unwrap());
        // }

        let signal = signal_color_moving_avg > BIT_THRESHOLD;
        let clock = clock_color_moving_avg > BIT_THRESHOLD;
        let signal_high = signal_color_moving_avg > 240;
        let clock_high = clock_color_moving_avg > 240;

        let desc = if signal_high && clock_high {
                Desc::Both
        } else if signal && clock {
            if signal_color_moving_avg > clock_color_moving_avg {
                Desc::Left
            } else {
                Desc::Right
            }
        } else {
            Desc::Neither
        };

        let freq_desc = self.most_freq_desc.push(desc);


        // eprintln!("\r{:?} {:?} {} {} {} {} {} {} {} {}",
        //     desc, freq_desc, signal_color_moving_avg, clock_color_moving_avg, top_color, bottom_color,
        //     signal, clock, signal_high, clock_high
        // );

        eprintln!("\r{:?}\tPrev Clock: {}", freq_desc, self.prev_clock);

        let (actual_signal, actual_clock) = match freq_desc {
            Desc::Left => (true, false),
            Desc::Right => (false, true),
            Desc::Both => (true, true),
            Desc::Neither => (false, false)
        };

        let result = if actual_clock != self.prev_clock {
            eprintln!("{} {}", actual_signal, actual_clock);

            self.bits[0] = actual_signal;
            Some(&self.bits)
        } else {
            None
        };

        self.prev_clock = actual_clock;

        result
    }
}

#[derive(Clone,Copy,Debug,Eq,Hash,PartialEq)]
enum Desc2 {
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
    most_freq_desc: MostFrequent<Desc2>
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

        // let bottom_left: (u8,u8,u8) = image.at2(0, 0);
        // let bottom_right: (u8,u8,u8) = image.at2(0, size.height as isize);
        // let top_left: (u8,u8,u8) = image.at2(size.width as isize, 0);
        // let top_right: (u8,u8,u8) = image.at2(size.width as isize, size.height as isize);
        //
        // eprintln!("Element size: {}", image.elem_size());
        // eprintln!("Corners: {:?} {:?} {:?} {:?}", bottom_left, bottom_right, top_left, top_right);

        // let left_rgb = sample_color(&image, 0, (size.width / 2) as usize, 0, size.height as usize);
        // let right_rgb = sample_color(&image, 0, (size.width / 2) as usize, 0, size.height as usize);
        // let left_rgb = sample_color(&image, 0, size.height as usize, 0, (size.width / 2) as usize);
        // let right_rgb = sample_color(&image, 0, size.height as usize, 0, (size.width / 2) as usize);
        // eprintln!("---TOP---");
        let top_rgb = sample_color(&image, 0, size.width as usize, (size.height - 100) as usize, size.height as usize);
        // eprintln!("---BOTTOM---");
        let bottom_rgb = sample_color(&image, 0, size.width as usize, 0, 100);
        // eprintln!("---LEFT---");
        let left_rgb = sample_color(&image, 0, 100, 0, size.height as usize);
        // eprintln!("---RIGHT---");
        let right_rgb = sample_color(&image, (size.width - 100) as usize, size.width as usize, 0, size.height as usize);
        // eprintln!("left: {:?}\tright: {:?}", left_rgb, right_rgb);

        let rgb = sample_color(image, 0, size.width as usize, 0, size.height as usize);

        let g_avg = self.left_g_avg.push(left_rgb[1] as f64);
        let r_avg = self.right_r_avg.push(right_rgb[0] as f64);

        // let r_avg = self.r_avg.push(rgb[0] as f64);
        // let g_avg = self.g_avg.push(rgb[1] as f64);

        let desc = if r_avg < 20f64 && g_avg < 20f64 {
            Desc2::Neither
        } else if r_avg > 60f64 && g_avg > 60f64 {
            Desc2::Both
        } else if r_avg > g_avg {
            Desc2::Clock
        } else {
            Desc2::Signal
        };

        let desc = self.most_freq_desc.push(desc);
        // eprintln!("width: {} height: {}", size.width, size.height);
        // eprintln!("{:?} {:?} {:?} {:?} {:?} {:?}", desc, rgb, top_rgb, bottom_rgb, left_rgb, right_rgb);

        let (signal, clock) = match desc {
            Desc2::Signal => (true, false),
            Desc2::Clock => (false, true),
            Desc2::Both => (true, true),
            Desc2::Neither => (false, false)
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
