extern crate rust_pigpio;
extern crate i2cdev;
extern crate byteorder;
extern crate gilrs;

extern crate robot;

use std::{thread, time};
use rust_pigpio::*;

use gilrs::{Gilrs, Button, Event };
use gilrs::Axis::{LeftZ, RightZ, LeftStickX, LeftStickY, DPadY};

use robot::servo::*;
use robot::motor::*;
use robot::ssd1327::*;
//use robot::hmc5883l::*;   Not connected atm 
use robot::vl53l0x::*;

fn main() {
		
	// Test OLED display
	let mut display = SSD1327::new("/dev/i2c-9");
	display.begin().unwrap();
	display.clear();
	display.draw_text(20, 42, "Forest", WHITE).unwrap();
	display.draw_text(20, 50, "Fighters", WHITE).unwrap();
	display.draw_text(20, 58, "Ready...", WHITE).unwrap();
	display.update_all().unwrap();	
	
	// Test compass Not connected atm 
	//let mut compass = HMC5883L::new("/dev/i2c-9").unwrap();
	//println!("Current Heading {:.*}", 1, compass.read_degrees().unwrap());
	
	// Test distance sensors
	let mut front = VL53L0X::new( "/dev/i2c-5").unwrap();
	let mut right = VL53L0X::new( "/dev/i2c-6").unwrap();
	let mut left = VL53L0X::new( "/dev/i2c-7").unwrap();
	let mut back = VL53L0X::new( "/dev/i2c-8").unwrap(); 
	
	println!("Front Distance {:.*}", 1, front.read().unwrap());
	println!("Right Distance {:.*}", 1, right.read().unwrap());
	println!("Left Distance {:.*}", 1, left.read().unwrap());
	println!("Back Distance {:.*}", 1, back.read().unwrap());
	
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
