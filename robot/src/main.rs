extern crate rust_pigpio;
extern crate i2cdev;
extern crate byteorder;
extern crate gilrs;

use std::{thread, time};
use std::time::Duration;
use std::cmp::{min, max};
use rust_pigpio::*;
use rust_pigpio::pwm::*;

use i2cdev::linux::*;
use i2cdev::core::*;

use gilrs::{Gilrs, Button, Event };
use gilrs::Axis::{LeftZ, RightZ, LeftStickX, LeftStickY, DPadY};

struct Servo {
	pwm_pin: u32
}

impl Servo {	
	
	fn init( &self ){				
		set_mode(self.pwm_pin, OUTPUT).unwrap();		
		servo(self.pwm_pin, 0);
		//set_pwm_frequency(self.pwm_pin, 500).unwrap();
		//set_pwm_range(self.pwm_pin, 1000).unwrap();
	}
	
	fn set_pulse_width( &self, mut width: u32 ) {
		if width < 500  {
			width = 500;
		}
		if width > 2500 {
			width = 2500;
		}
		servo(self.pwm_pin, width);
	}	
		
}

fn build_servo( pwm_pin: u32 ) -> Servo {
	Servo {
		pwm_pin		
	}
}

struct Motor {			
	pwm_pin: u32,
	dir_pin: u32,	
}

impl Motor {	
	
	fn init( &self ){				
		set_mode(self.dir_pin, OUTPUT).unwrap();
		set_mode(self.pwm_pin, OUTPUT).unwrap();
		write(self.dir_pin, ON).unwrap();
		set_pwm_frequency(self.pwm_pin, 500).unwrap(); 	// Set to modulate at 500hz.
		set_pwm_range(self.pwm_pin, 1000).unwrap(); 		// Set range to 1000. 1 range = 2 us;
	}
	
	fn power( &self, power: i32 ) {
		
		// limit range
		let actual = min(1000, power.abs());
		
		if power >= 0 {			
			let pwm = actual as u32;
			//println!("Forward Power {0} ", pwm);
			self.forward( pwm );
		}
		else {
			let pwm = actual as u32;
			//println!("Backward Power {0} ", pwm);
			self.backward( pwm );
		}
	}

	fn forward( &self, power: u32 ) {		
		write(self.dir_pin, ON).unwrap();	
		pwm(self.pwm_pin, power).unwrap();			
	}mecanum
	
	fn backward( &self, power: u32 ) {		
		write(self.dir_pin, OFF).unwrap();	
		pwm(self.pwm_pin, power).unwrap();					
	}
	
	fn stop( &self ){				
		write(self.dir_pin, ON).unwrap();	
		pwm(self.pwm_pin, 0).unwrap();							
	}

}

fn build_motor( pwm_pin: u32, dir_pin: u32 ) -> Motor {
	Motor {
		pwm_pin,
		dir_pin
	}
}

fn main() {
    println!("Initialized pigpio. Version: {}", initialize().unwrap());
    let interval = time::Duration::from_millis(2000);

    //Use BCM numbering
    let left_rear_motor = build_motor( 17, 27);   //( 11, 15);       
	left_rear_motor.init();
		
	let right_rear_motor = build_motor( 25, 23);  //( 22, 16);       	
	right_rear_motor.init();
	
	let left_front_motor = build_motor( 10, 11);  //( 19, 21);		
	left_front_motor.init();
	
	let right_front_motor = build_motor( 12, 8);  //( 32, 24);	
	right_front_motor.init();
	
	let servo = build_servo( 9 );
	    
    let mut gilrs = Gilrs::new().unwrap();
    
    let mut lock = true;
    let mut gear = 1;

	// Iterate over all connected gamepads
	for (_id, gamepad) in gilrs.gamepads() {
		println!("{} is {:?}", gamepad.name(), gamepad.power_info());
	}    
    
    loop {
		while let Some(Event { id, event, time }) = gilrs.next_event() {
			println!("{:?} New event from {}: {:?}", time, id, event); 
			break;				
		}
					
		let mut left_stick_x = 0;
		let mut left_stick_y = 0;
		let mut right_stick_y = 0;
		let mut right_stick_x = 0;
		
		let mut dpad = 0;
				
		if gilrs[0].axis_data(LeftStickY).is_some() {				
			left_stick_y = (gilrs[0].axis_data(LeftStickY).unwrap().value() * 1000.0) as i32;
		}
		
		if gilrs[0].axis_data(LeftStickX).is_some() {				
			left_stick_x = (gilrs[0].axis_data(LeftStickX).unwrap().value() * 1000.0) as i32;
		}
		
		if gilrs[0].axis_data(RightZ).is_some() {				
			right_stick_y = (gilrs[0].axis_data(RightZ).unwrap().value() * -1000.0) as i32;
		}
		
		if gilrs[0].axis_data(LeftZ).is_some() {				
			right_stick_x = (gilrs[0].axis_data(LeftZ).unwrap().value() * 1000.0) as i32;	
		}	
		
		if gilrs[0].axis_data(DPadY).is_some() {				
			dpad = (gilrs[0].axis_data(DPadY).unwrap().value()) as i32;
		}	
		
		if gilrs[0].is_pressed(Button::LeftTrigger2) {
			lock = true;
		}
		
		if gilrs[0].is_pressed(Button::RightTrigger2) {
			lock = false;
		}
		
		if gilrs[0].is_pressed(Button::North) {
			gear = 1;
			println!(" {0} ",gear);			
		}
		
		if gilrs[0].is_pressed(Button::West) {
			gear = 2;
			println!(" {0} ",gear);			
		}
		
		if gilrs[0].is_pressed(Button::East) {
			gear = 3;			
			println!(" {0} ",gear);
		}
		
		if gilrs[0].is_pressed(Button::South) {
			gear = 4;			
			println!(" {0} ",gear);
		}
					
		if gilrs[0].is_pressed(Button::West) && gilrs[0].is_pressed(Button::South) {
			break;
		}
		
		//if left_stick_Y != 0 || bwd != 0 || left != 0 || right != 0  {
			//println!(" {0}, {1}, {2}, {3} ", left_stick_Y, bwd, left, right );
		//}		
		
		let mut left_rear_speed: i32;
		let mut right_rear_speed: i32;
		let mut left_front_speed: i32;
		let mut right_front_speed: i32;
		
		if lock {			
			if left_stick_y == 0 && right_stick_y == 0 {				
				left_rear_speed = 0;
				right_rear_speed = 0;
				left_front_speed = 0;
				right_front_speed = 0;
			}
			else
			{
				left_front_speed  = left_stick_y;
				left_rear_speed   = left_stick_y;
				right_front_speed = -right_stick_y;			
				right_rear_speed  = -right_stick_y;
			}
		}
		else
		{
			if left_stick_y == 0 && left_stick_x == 0 {				
				left_rear_speed = 0;
				right_rear_speed = 0;
				left_front_speed = 0;
				right_front_speed = 0;
			}
			else
			{			
				left_front_speed  = -left_stick_x + left_stick_y - right_stick_x;				
				left_rear_speed   = left_stick_x + left_stick_y - right_stick_x;
				
				right_rear_speed  = -left_stick_x - left_stick_y + right_stick_x;
				right_front_speed = left_stick_x - left_stick_y + right_stick_x;
			}
		}
		
		if dpad == 1 {
			servo.set_pulse_width( 2500 );
		}
		else if dpad == -1 {
			servo.set_pulse_width( 500 );
		}
		
		left_front_speed  = left_front_speed / gear;
		right_front_speed = right_front_speed / gear;
		left_rear_speed   = left_rear_speed / gear;
		right_rear_speed  = right_rear_speed / gear;		
		
		if left_rear_speed != 0 || right_rear_speed != 0 || left_front_speed != 0 || right_front_speed != 0  {
			println!(" {0}, {1}, {2}, {3} ", left_rear_speed, right_rear_speed, left_front_speed, right_front_speed );
		}	
		
		left_rear_motor.power(left_rear_speed);
		right_rear_motor.power(right_rear_speed);	
		left_front_motor.power(left_front_speed);
		right_front_motor.power(right_front_speed);	
			
	}
    
    left_rear_motor.stop();    
    right_rear_motor.stop();
    left_front_motor.stop();
    right_front_motor.stop();
    
    thread::sleep(interval);    
    
    terminate();
}
