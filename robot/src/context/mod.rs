extern crate gilrs;

use self::gilrs::Gilrs;
use pixel::*;
use ssd1327::*;

pub struct Context {
    pub display: SSD1327,
    pub gilrs: Gilrs,
    pub pixel: Pixel,
}

impl Context {
    pub fn new(filename: &'static str) -> Context {
        Context {
            display: SSD1327::new(filename),
            gilrs: Gilrs::new().unwrap(),
            pixel: build_pixel(),
        }
    }
}
