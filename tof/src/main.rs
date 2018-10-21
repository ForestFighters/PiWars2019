//extern crate rust_pigpio;
use std::{thread, time};

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
        
        let revision = tof.smbus_read_byte_data(VL53L0X_REG_IDENTIFICATION_REVISION_ID).unwrap();      
		let model = tof.smbus_read_byte_data(VL53L0X_REG_IDENTIFICATION_MODEL_ID).unwrap();
		println! ("Revision: {0} Model {1}", revision, model);
		
		let prerange = tof.smbus_read_byte_data(VL53L0X_REG_PRE_RANGE_CONFIG_VCSEL_PERIOD).unwrap();
		println! ("PRE_RANGE_CONFIG_VCSEL_PERIOD = {0}",prerange);

		let range = tof.smbus_read_byte_data(VL53L0X_REG_FINAL_RANGE_CONFIG_VCSEL_PERIOD).unwrap();
		println!("FINAL_RANGE_CONFIG_VCSEL_PERIOD = {0}",range);

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

		if (status & 0x01) == 0x01 {
			println!("ready");
		}
		else {
			println!( "not ready");
		}

		let data = self.tof.smbus_read_i2c_block_data(VL53L0X_REG_RESULT_RANGE_STATUS, 12).unwrap();
		//println!("{:#?}",data);

		let dist1:u16 = (data[10]).into();
		let dist2:u16 = (data[11]).into();
		let distance = (dist1 * 256) + dist2;
		//println!("distance {:#?}mm",distance);
        // return distance
        Ok(distance as u16)
    }
}
// ---------------------------------- 8< ---------------------------------------------------------

//mod VL53L0X;

fn main() {

	const ADDRESS: u16										= 0x29;
	
	let device = "/dev/i2c-1";
	let mut tof = VL53L0X::new( device, ADDRESS).unwrap();
	
	loop {
		thread::sleep(time::Duration::from_millis(500));
		let distance = tof.read().unwrap();
		println!("distance {:#?}mm",distance);		
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
