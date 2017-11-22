extern crate cv;
extern crate rand;

// use std::time::{Duration,Instant};

use cv::highgui::{WindowFlags,highgui_named_window};
use cv::videoio::{CapProp,VideoCapture};



mod decode;
mod util;

use self::decode::{Decoder,TimedColorCodedOneBitDecoder};


const FPS: u8 = 60;


// fn as_f64(duration: &Duration) -> f64 {
//     duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9
// }

fn main() {
    let cap = VideoCapture::new(2);
    assert!(cap.is_open());

    cap.set(CapProp::FrameWidth, 320f64);
    cap.set(CapProp::FrameHeight, 240f64);
    cap.set(CapProp::Fps, FPS as f64);

    eprintln!("Width: {}", cap.get(CapProp::FrameWidth).unwrap());
    eprintln!("Height: {}", cap.get(CapProp::FrameHeight).unwrap());
    eprintln!("FPS: {}", cap.get(CapProp::Fps).unwrap());

    highgui_named_window("Window", WindowFlags::WindowAutosize);

    // let mut decoder = GreedyOneBitDecoder::new();
    // let mut decoder = TimedOneBitDecoder::new();
    let mut decoder = TimedColorCodedOneBitDecoder::new();

    // let start_time = Instant::now();
    // // let mut prev_time = start_time;
    // let mut prev_print_time = start_time;

    // let mut frames: u64 = 0;

    let mut byte_in_progress: Vec<bool> = Vec::new();

    while let Some(image) = cap.read() {
        // frames += 1;

        image.show("Window", 1).unwrap();

        if let Some(bits) = decoder.decode(&image) {
            for bit in bits {
                byte_in_progress.push(*bit);

                if byte_in_progress.len() == 8 {
                    let mut byte = 0u8;
                    for idx in 0..8 {

                        byte <<= 1;

                        if byte_in_progress[idx] {
                            byte |= 1;
                        }

                    }

                    eprintln!("Byte: {} {}", byte, char::from(byte));
                    print!("{}", byte);
                    byte_in_progress.clear();
                }
            }
        }

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

// mod at_test;
// fn main() {
//     at_test::test_at();
// }
