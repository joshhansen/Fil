use std::fs::File;
use std::io::Read;

use cv::Mat;
use cv::imgcodecs::ImreadModes;

pub fn test_at() {
    let bytes: Vec<u8> = File::open("test/red_10x10.png").unwrap()
        .bytes().map(|byte_result| byte_result.unwrap()).collect();

    let mat = Mat::imdecode(&bytes, ImreadModes::ImreadColor);

    let size = mat.size();
    for x in 0..size.width {
        for y in 0..size.height {
            let rgb: (u8,u8,u8) = mat.at2(x as isize, y as isize);
            println!("{},{} -> {:?}", x, y, rgb);
        }
    }
}
