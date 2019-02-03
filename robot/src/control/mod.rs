extern crate byteorder;
extern crate i2cdev;
extern crate rust_pigpio;

use self::rust_pigpio::*;
use motor::*;
use std::{thread, time};

pub struct Control {
    pub gear: i32,
    // Channel 4
    pub left_rear_motor: Motor,
    // Channel 3
    pub right_rear_motor: Motor,
    // Channel 2
    pub left_front_motor: Motor,
    // Channel 1
    pub right_front_motor: Motor,
    // Bias
    pub bias: i32,
}

impl Control {
    pub fn init(&self) {
        println!("Initialized pigpio. Version: {}", initialize().unwrap());
        self.left_rear_motor.init();
        self.right_rear_motor.init();
        self.left_front_motor.init();
        self.right_front_motor.init();
        self.stop();
    }

    pub fn set_gear(&mut self, gear: i32) {
        self.gear = gear;
    }

    pub fn set_bias(&mut self, bias: i32) {
        self.bias = bias;
    }

    pub fn speed(
        &self,
        left_rear_speed: i32,
        right_rear_speed: i32,
        left_front_speed: i32,
        right_front_speed: i32,
    ) {
        let lr_speed = (left_rear_speed + self.bias) / self.gear;
        let rr_speed = right_rear_speed / self.gear;
        let lf_speed = (left_front_speed + self.bias) / self.gear;
        let rf_speed = right_front_speed / self.gear;

        self.left_rear_motor.power(lr_speed);
        self.right_rear_motor.power(rr_speed);
        self.left_front_motor.power(lf_speed);
        self.right_front_motor.power(rf_speed);
    }

    pub fn stop(&self) {
        self.left_rear_motor.stop();
        self.right_rear_motor.stop();
        self.left_front_motor.stop();
        self.right_front_motor.stop();
    }

    pub fn drive(&mut self, speed: i32, gear: i32) {
        self.set_gear(gear);
        self.speed(speed, speed * -1, speed, speed * -1);
    }

    pub fn turn_left(&mut self, speed: i32, gear: i32) {
        self.set_gear(gear);
        self.speed(speed * -1, speed * -1, speed * -1, speed * -1);
    }

    pub fn turn_right(&mut self, speed: i32, gear: i32) {
        self.set_gear(gear);
        self.speed(speed, speed, speed, speed);
    }
}

impl Drop for Control {
    fn drop(&mut self) {
        println!("Terminate pigpio");
        terminate();
        let interval = time::Duration::from_millis(2000);
        thread::sleep(interval);
    }
}

pub fn build_control() -> Control {
    let gear = 3;

    let left_rear_motor = build_motor(10, 11);

    let right_rear_motor = build_motor(9, 8);

    let left_front_motor = build_motor(15, 22);

    let right_front_motor = build_motor(14, 27);

    let bias = 0;

    Control {
        gear,
        left_rear_motor,
        right_rear_motor,
        left_front_motor,
        right_front_motor,
        bias,
    }
}
