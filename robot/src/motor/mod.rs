extern crate byteorder;
extern crate i2cdev;
extern crate rust_pigpio;

use self::rust_pigpio::pwm::*;
use self::rust_pigpio::*;

use std::cmp::min;

pub struct Motor {
    pub pwm_pin: u32,
    pub dir_pin: u32,
}

impl Motor {
    pub fn init(&self) {
        set_mode(self.dir_pin, OUTPUT).unwrap();
        set_mode(self.pwm_pin, OUTPUT).unwrap();
        write(self.dir_pin, ON).unwrap();
        set_pwm_frequency(self.pwm_pin, 500).unwrap(); // Set to modulate at 500hz.
        set_pwm_range(self.pwm_pin, 1000).unwrap(); // Set range to 1000. 1 range = 2 us;
    }

    pub fn power(&self, power: i32) {
        // limit range
        let actual = min(1000, power.abs());

        if power >= 0 {
            let pwm = actual as u32;
            //println!("Forward Power {0} ", pwm);
            self.forward(pwm);
        } else {
            let pwm = actual as u32;
            //println!("Backward Power {0} ", pwm);
            self.backward(pwm);
        }
    }

    pub fn forward(&self, power: u32) {
        write(self.dir_pin, ON).unwrap();
        pwm(self.pwm_pin, power).unwrap();
    }

    pub fn backward(&self, power: u32) {
        write(self.dir_pin, OFF).unwrap();
        pwm(self.pwm_pin, power).unwrap();
    }

    pub fn stop(&self) {
        write(self.dir_pin, ON).unwrap();
        pwm(self.pwm_pin, 0).unwrap();
    }
}

pub fn build_motor(pwm_pin: u32, dir_pin: u32) -> Motor {
    Motor { pwm_pin, dir_pin }
}
