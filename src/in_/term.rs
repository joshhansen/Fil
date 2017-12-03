use std::collections::{HashMap,VecDeque};
use std::io::{Write,stdout};

use cv::Mat;
use cv::highgui::{WindowFlags,highgui_named_window};
use cv::videoio::{CapProp,VideoCapture};

use rand;
use rand::distributions::{IndependentSample, Range};

use super::super::util::{MovingAvg,MostFrequent};

const FPS: u8 = 60;
const SAMPLES: u64 = 1000;


pub trait VideoDecoder {
    /// FIXME: When const generics happen, use a parameterized array rather than vector
    fn decode_video(&mut self, image: &Mat) -> Option<&Vec<bool>>;
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
impl VideoDecoder for TimedColorCodedOneBitDecoder {
    fn decode_video(&mut self, image: &Mat) -> Option<&Vec<bool>> {
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
            // eprintln!("{} {}", signal, clock);

            self.bits[0] = signal;
            Some(&self.bits)
        } else {
            None
        };

        self.prev_clock = clock;

        result
    }
}

struct GridDecoder {
    bits: Vec<bool>,
    prev_clock: bool,
}

impl GridDecoder {
    fn new(grid_height: usize, grid_width: usize) -> Self {
        Self {
            bits: Vec::with_capacity(grid_height * grid_width),
            prev_clock: false
        }
    }
}

/// From https://en.wikipedia.org/w/index.php?title=Connected-component_labeling&oldid=801482060#One_component_at_a_time
fn connected_components(image: &Mat) {
    let mut components: HashMap<(isize,isize),usize> = HashMap::new();
    let mut q: VecDeque<(isize,isize)> = VecDeque::new();

    let size = image.size();

    let mut component_num = 0;

    for x in (0 as isize)..(size.width as isize) {
        for y in (0 as isize)..(size.height as isize) {
            let (g,b,r): (u8,u8,u8) = image.at2(x,y);
            if r > 200 && g < 50 && b < 50 {
                components.insert((x,y), component_num);
                q.push_back((x,y));
            }
        }
    }
}

impl VideoDecoder for GridDecoder {
    fn decode_video(&mut self, image: &Mat) -> Option<&Vec<bool>> {
        None
    }
}

pub fn decode<F:Fn(Option<&Vec<u8>>)>(callback: F) {
    let mut result: Vec<u8> = Vec::with_capacity(1);

    let cap = VideoCapture::new(1);
    assert!(cap.is_open());

    cap.set(CapProp::FrameWidth, 320f64);
    cap.set(CapProp::FrameHeight, 240f64);
    cap.set(CapProp::Fps, FPS as f64);

    eprintln!("Width: {}", cap.get(CapProp::FrameWidth).unwrap());
    eprintln!("Height: {}", cap.get(CapProp::FrameHeight).unwrap());
    eprintln!("FPS: {}", cap.get(CapProp::Fps).unwrap());

    highgui_named_window("Window", WindowFlags::WindowAutosize);

    let mut decoder = TimedColorCodedOneBitDecoder::new();

    // let start_time = Instant::now();
    // // let mut prev_time = start_time;
    // let mut prev_print_time = start_time;

    // let mut frames: u64 = 0;

    let mut byte_in_progress: Vec<bool> = Vec::new();

    while let Some(image) = cap.read() {
        // frames += 1;

        image.show("Window", 1).unwrap();

        if let Some(bits) = decoder.decode_video(&image) {
            for bit in bits {
                byte_in_progress.push(*bit);

                if byte_in_progress.len() == 8 {
                    byte_in_progress.reverse();
                    let mut byte = 0u8;
                    for idx in 0..8 {

                        byte <<= 1;

                        if byte_in_progress[idx] {
                            byte |= 1;
                        }

                    }

                    result[0] = byte;
                    callback(Some(&result));

                    let c = char::from(byte);
                    eprintln!("Byte: {} {}", byte, c);
                    print!("{}", c);
                    stdout().flush().unwrap();
                    byte_in_progress.clear();
                }
            }
        }

        callback(None);
        // let new_time = Instant::now();
        // let avg_fps = frames as f64 / as_f64( &new_time.duration_since(start_time) );
        // let elapsed_since_printing = as_f64(&new_time.duration_since(prev_print_time));
        //
        // // prev_time = new_time;
        //
        // if elapsed_since_printing > 1.0 {
        //     // print!("\r{} fps", avg_fps);
        //     // stdout().flush().unwrap();
        //     prev_print_time = new_time;
        // }
    }
}
