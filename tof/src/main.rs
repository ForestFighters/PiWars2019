extern crate rust_pigpio;
use std::{thread, time};
use std::cmp::{min, max};

// ---------------------------------- 8< ---------------------------------------------------------
use rust_pigpio::*;
use rust_pigpio::pwm::*;

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
	}
	
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

// ---------------------------------- 8< ---------------------------------------------------------
extern crate i2cdev;
extern crate byteorder;
use i2cdev::linux::*;
use i2cdev::core::I2CDevice;

const VL53L0X_REG_IDENTIFICATION_MODEL_ID: u8			= 0xc0;
const VL53L0X_REG_IDENTIFICATION_REVISION_ID: u8	 	= 0xc2;
const VL53L0X_REG_PRE_RANGE_CONFIG_VCSEL_PERIOD: u8	 	= 0x50;
const VL53L0X_REG_FINAL_RANGE_CONFIG_VCSEL_PERIOD: u8 	= 0x70;
const VL53L0X_REG_SYSRANGE_START: u8					= 0x00;

//const VL53L0X_REG_RESULT_INTERRUPT_STATUS: u8			= 0x13;
const VL53L0X_REG_RESULT_RANGE_STATUS: 	u8				= 0x14;

pub struct VL53L0X {
    tof: Box<LinuxI2CDevice>
}

impl VL53L0X {

    pub fn new(filename: &'static str, address: u16) -> Result<Self, Box<LinuxI2CError>> {

        let mut tof = try!(LinuxI2CDevice::new(filename, address));
        // delay before taking first reading
        thread::sleep(time::Duration::from_millis(100));
        
        let _revision = tof.smbus_read_byte_data(VL53L0X_REG_IDENTIFICATION_REVISION_ID).unwrap();      
		let _model = tof.smbus_read_byte_data(VL53L0X_REG_IDENTIFICATION_MODEL_ID).unwrap();
		//println! ("Revision: {0} Model {1}", revision, model);
		
		let _prerange = tof.smbus_read_byte_data(VL53L0X_REG_PRE_RANGE_CONFIG_VCSEL_PERIOD).unwrap();
		//println! ("PRE_RANGE_CONFIG_VCSEL_PERIOD = {0}",prerange);

		let _range = tof.smbus_read_byte_data(VL53L0X_REG_FINAL_RANGE_CONFIG_VCSEL_PERIOD).unwrap();
		//println!("FINAL_RANGE_CONFIG_VCSEL_PERIOD = {0}",range);

        Ok(VL53L0X { tof: Box::new(tof) })
    }

    pub fn read(&mut self) -> Result<(u16), Box<LinuxI2CError>> {
		let interval = time::Duration::from_millis(10);
        let mut cnt = 0;
		let _start = self.tof.smbus_write_byte_data(VL53L0X_REG_SYSRANGE_START, 0x01);
		let mut status = self.tof.smbus_read_byte_data(VL53L0X_REG_RESULT_RANGE_STATUS).unwrap();
		loop {
			if (status & 0x01) == 0x01 || cnt >= 100  {
				break;
			}
			// 1 second waiting time max
			thread::sleep(interval);    
			status = self.tof.smbus_read_byte_data(VL53L0X_REG_RESULT_RANGE_STATUS).unwrap();
			cnt += 1;
		}

		if (status & 0x01) != 0x01 {		
			println!( "not ready");
		}

		let data = self.tof.smbus_read_i2c_block_data(VL53L0X_REG_RESULT_RANGE_STATUS, 12).unwrap();
		//println!("{:#?}",data);

		let dist1:u16 = (data[10]).into();
		let dist2:u16 = (data[11]).into();
		let mut distance = (dist1 * 256) + dist2;
		//println!("distance {:#?}mm",distance);
        // return distance
        if distance <= 20 || distance > 1900 {
			distance = 9999
		}
        Ok(distance as u16)
    }
}
// ---------------------------------- 8< ---------------------------------------------------------

//mod VL53L0X;

//1 Go forwards until front sensor is reading ~150mm  and the rhs reads ~150mm
//2 Go left until left sensor is reading 150mm and the front sensor reads ~150mm
//3 Go back until back sensor is reading 150mm and left reads 750mm and right reads 150mm
//4 Go left until left sensor is reading 150mm and back reads 150mm and right reads 750mm
//5 Go forwards until front sensor is reading 150mm and the left reads 750mm and the right reads 150mm
//6 Go left until left sensor is reading 150mm and the front is reading 150mm and the right reads 750mm
//7 Go back until back sensor is reading 150mm and the front is reading ??? and the left reads 150mm
//8 Go right until the right sensor is reading 150mm and front is reading 750mm and the left reads ????mm
//9 Go back until the front sensor reads 1500mm


fn drive( direction: &str  )
{
	println!("{}[u", 27 as char);
	println!("Direction {:#?}",direction);		
}

fn main() {

	println!("Initialized pigpio. Version: {}", initialize().unwrap());
    let interval = time::Duration::from_millis(2000);

    //Use BCM numbering
    let left_rear_motor = build_motor( 17, 27);
	left_rear_motor.init();
		
	let right_rear_motor = build_motor( 25, 23);
	right_rear_motor.init();
	
	let left_front_motor = build_motor( 10, 11);
	left_front_motor.init();
	
	let right_front_motor = build_motor( 12, 8);
	right_front_motor.init();
	
	const ADDRESS: u16										= 0x29;
	const SPEED: i32 										= 160;
		
	let mut front = VL53L0X::new( "/dev/i2c-5", ADDRESS).unwrap();
	let mut right = VL53L0X::new( "/dev/i2c-6", ADDRESS).unwrap();
	let mut left = VL53L0X::new( "/dev/i2c-7", ADDRESS).unwrap();
	let mut back = VL53L0X::new( "/dev/i2c-8", ADDRESS).unwrap();
	
	let mut distance: u16 = 0;
	let mut direction = "Front";
	let mut state = 1;	
	
	let mut left_rear_speed: i32;
	let mut right_rear_speed: i32;
	let mut left_front_speed: i32;
	let mut right_front_speed: i32;
	
	println!("{}c", 27 as char);
	println!("{}[s", 27 as char);
	loop {
		//thread::sleep(time::Duration::from_millis(100));
		let front_dist = front.read().unwrap();
		let right_dist = right.read().unwrap();
		let left_dist = left.read().unwrap();
		let back_dist = back.read().unwrap();
		
		println!("{}[u", 27 as char);
		println!("Direction {:#?}, Distance {:#?}mm      ",direction, distance);	
		
		if direction == "Front" {
			left_rear_speed =    SPEED;
			right_rear_speed =   SPEED * -1;
			left_front_speed =   SPEED;
			right_front_speed =  SPEED * -1;
		}
		else if direction == "Back" {
			left_rear_speed =    SPEED  * -1;
			right_rear_speed =   SPEED;
			left_front_speed =   SPEED  * -1;
			right_front_speed =  SPEED;
		}
		else if direction == "Left" {
			left_rear_speed =   SPEED * -1;
			right_rear_speed =  SPEED;
			left_front_speed =  SPEED;
			right_front_speed = SPEED * -1;
		}
		else if direction == "Right" {
			left_rear_speed =   SPEED;
			right_rear_speed =  SPEED * -1;
			left_front_speed =  SPEED * -1;
			right_front_speed = SPEED;
		}
		else {
			left_rear_speed =  0;
			right_rear_speed = 0;
			left_front_speed = 0;
			right_front_speed = 0;			
		}
		
		left_rear_motor.power(left_rear_speed);
		right_rear_motor.power(right_rear_speed);	
		left_front_motor.power(left_front_speed);
		right_front_motor.power(right_front_speed);	
		//println!("front {:#?}mm      ",front_dist);		
		//println!("right {:#?}mm      ",right_dist);	
		//println!("left {:#?}mm       ",left_dist);	
		//println!("back {:#?}mm       ",back_dist);	
		
		if state == 1 && front_dist < 150  {
			// && right_dist < 150
			state = 2;
			direction = "Left";
			distance = front_dist;
		}
		if state == 2 && left_dist < 150  {
			// && front_dist < 150
			state = 3;
			direction = "Back";
			distance = left_dist;
		}
		if state == 3 && back_dist < 150  {
			// && right_dist < 150 && left_dist > 750
			state = 4;
			direction = "Left";
			distance = back_dist;
		}
		if state == 4 && left_dist < 150 {
			// && back_dist < 150 && right_dist > 750
			state = 5;
			direction = "Front";
			distance = left_dist;
		}
		if state == 5 && front_dist < 150 {
			// && right_dist < 150 && left_dist > 750
			state = 6;
			direction = "Left";
			distance = front_dist;			
		}
		if state == 6 && left_dist < 150  {
			// && front_dist < 150 && right_dist > 750
			state = 7;
			direction = "Back";
			distance = left_dist;
		}
		if state == 7 && back_dist < 150 {
			// && left_dist < 150 && front_dist > 750
			state = 8;
			direction = "Right";
			distance = back_dist;
		}
		if state == 8 && right_dist < 150  {
			// && left_dist > 750 && front_dist > 750
			state = 9;
			direction = "Back";
			distance = right_dist;
		}
		if state == 9 && front_dist > 2000 {
			state = 10;
			direction = "Finished";	
			distance = front_dist;
		}
		if state == 10 {			
			break;
		}
	}
}

//fn main() {
	//let interval = time::Duration::from_millis(10);
    //println!("Hello, Amy! How are you, today?");
    
    //let bus = "/dev/i2c-1";
    //let mut tof = LinuxI2CDevice::new(bus,ADDRESS).unwrap();
    
    //let phalanges = tof.smbus_read_byte_data(VL53L0X_REG_IDENTIFICATION_REVISION_ID).unwrap();
   
    
    //let pickle = tof.smbus_read_byte_data(VL53L0X_REG_IDENTIFICATION_MODEL_ID).unwrap();
	//println! ("The happiest device ID: {0} {1}", phalanges, pickle);
	
	
	//let prerange = tof.smbus_read_byte_data(VL53L0X_REG_PRE_RANGE_CONFIG_VCSEL_PERIOD).unwrap();
	//println! ("PRE_RANGE_CONFIG_VCSEL_PERIOD = {0}",prerange);


	//let range = tof.smbus_read_byte_data(VL53L0X_REG_FINAL_RANGE_CONFIG_VCSEL_PERIOD).unwrap();
	//println!("FINAL_RANGE_CONFIG_VCSEL_PERIOD = {0}",range);

	//loop {
		//let mut cnt = 0;
		//let _start = tof.smbus_write_byte_data(VL53L0X_REG_SYSRANGE_START, 0x01);
		//let mut status = tof.smbus_read_byte_data(VL53L0X_REG_RESULT_RANGE_STATUS).unwrap();
		//loop {
			//if (status & 0x01) == 0x01 || cnt >= 100  {
				//break;
			//}
			//// 1 second waiting time max
			//thread::sleep(interval);    
			//status = tof.smbus_read_byte_data(VL53L0X_REG_RESULT_RANGE_STATUS).unwrap();
			//cnt += 1;
		//}

		//if (status & 0x01) == 0x01 {
			//println!("ready");
		//}
		//else {
			//println!( "not ready");
		//}

		//let data = tof.smbus_read_i2c_block_data(VL53L0X_REG_RESULT_RANGE_STATUS, 12).unwrap();
		//println!("{:#?}",data);
		////println!("ambient count {:#?}",data[7] * 256 + data[6]);
		////println!("signal count {:#?}",data[9] * 256 + data[8]);
		//let dist1:u16 = (data[10]).into();
		//let dist2:u16 = (data[11]).into();
		//let distance = (dist1 * 256) + dist2;
		//println!("distance {:#?}mm",distance);

		////let device_range_status_internal = (data[0] & 0x78);
		////println!("{0}",device_range_status_internal);
		//thread::sleep(time::Duration::from_millis(500));
	//}
	
	//println! ("Amy is the best unicorn (sorry Chloe)");

//}
