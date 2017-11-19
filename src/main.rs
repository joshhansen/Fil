use std::time::Instant;

extern crate cv;
extern crate rand;

use cv::{Mat,Scalar};
use cv::highgui::*;
use cv::videoio::{CapProp,VideoCapture};

use rand::{ThreadRng,Rng};
use rand::distributions::{IndependentSample, Range};

enum Color {
    BLACK,
    WHITE,
    GREY
}

struct Decoder {

struct GreedyOneBitDecoder {
    bits: [bool; 1],
    rng: ThreadRng
}

const SAMPLES: u64 = 1000;

impl GreedyOneBitDecoder {
    fn new() -> Self {
        Self {
            bits: [false],
            rng: rand::thread_rng()
        }
    }

    fn sample_color(&mut self, image: &Mat, x_min: usize, x_max: usize, y_min: usize, y_max: usize) -> [u8; 3] {
        let x_range = Range::new(x_min, x_max);
        let y_range = Range::new(y_min, y_max);

        let mut totals: [u64; 3] = [0,0,0];
        for _ in 0..SAMPLES {
            let x = x_range.ind_sample(&mut self.rng);
            let y = y_range.ind_sample(&mut self.rng);

            let scalar = image.at(x, y);
            totals[0] += scalar.v0 as u64;
            totals[1] += scalar.v1 as u64;
            totals[2] += scalar.v2 as u64;
        }

        let averages: [u8; 3] = [(totals[0]/SAMPLES) as u8, (totals[1]/SAMPLES) as u8, (totals[2]/SAMPLES) as u8];
        averages
    }

    fn decode(&mut self, image: &Mat) -> &[bool; 1] {
        let size = image.size();
        let color = self.sample_color(image, 0, size.width as usize, 0, size.height as usize);
        let avg_color = ((color[0] as u32 + color[1] as u32 + color[2] as u32) / 3) as u8;

        self.bits[0] = avg_color > 127;

        &self.bits
    }
}



fn main() {
    let cap = VideoCapture::new(1);
    assert!(cap.is_open());

    cap.set(CapProp::FrameWidth, 320f64);
    cap.set(CapProp::FrameHeight, 240f64);
    cap.set(CapProp::Fps, 187f64);

    println!("Width: {}", cap.get(CapProp::FrameWidth).unwrap());
    println!("Height: {}", cap.get(CapProp::FrameHeight).unwrap());
    println!("FPS: {}", cap.get(CapProp::Fps).unwrap());
    highgui_named_window("Window", WindowFlags::WindowAutosize);

    let mut decoder = GreedyOneBitDecoder::new();

    let mut prev_time = Instant::now();
    while let Some(image) = cap.read() {
        image.show("Window", 30).unwrap();

        let bits = decoder.decode(&image);

        let new_time = Instant::now();

        let duration = new_time.duration_since(prev_time);

        let elapsed = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
        let rate = 1f64 / elapsed;
        println!("Elapsed: {:?}", elapsed);
        println!("{} fps", rate);
        println!("Bits: {:?}", bits);

        prev_time = new_time;

    }
}
