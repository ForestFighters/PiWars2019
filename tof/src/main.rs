extern crate rust_pigpio;
extern crate i2cdev;
extern crate byteorder;

use i2cdev::linux::*;
use i2cdev::core::I2CDevice;
use std::{thread, time};

const VL53L0X_REG_IDENTIFICATION_MODEL_ID: u8			= 0xc0;
const VL53L0X_REG_IDENTIFICATION_REVISION_ID: u8	 	= 0xc2;
const VL53L0X_REG_PRE_RANGE_CONFIG_VCSEL_PERIOD: u8	 	= 0x50;
const VL53L0X_REG_FINAL_RANGE_CONFIG_VCSEL_PERIOD: u8 	= 0x70;
const VL53L0X_REG_SYSRANGE_START: u8					= 0x00;

const VL53L0X_REG_RESULT_INTERRUPT_STATUS: u8			= 0x13;
const VL53L0X_REG_RESULT_RANGE_STATUS: 	u8				= 0x14;


const address: u16										= 0x29;


fn main() {
	let interval = time::Duration::from_millis(10);
    println!("Hello, Amy! How are you, today?");
    
    let bus = "/dev/i2c-1";
    let mut tof = LinuxI2CDevice::new(bus,address).unwrap();
    
    let phalanges = tof.smbus_read_byte_data(VL53L0X_REG_IDENTIFICATION_REVISION_ID).unwrap();
   
    
    let pickle = tof.smbus_read_byte_data(VL53L0X_REG_IDENTIFICATION_MODEL_ID).unwrap();
	println! ("The happiest device ID: {0} {1}", phalanges, pickle);
	
	
	let prerange = tof.smbus_read_byte_data(VL53L0X_REG_PRE_RANGE_CONFIG_VCSEL_PERIOD).unwrap();
	println! ("PRE_RANGE_CONFIG_VCSEL_PERIOD = {0}",prerange);


	let range = tof.smbus_read_byte_data(VL53L0X_REG_FINAL_RANGE_CONFIG_VCSEL_PERIOD).unwrap();
	println!("FINAL_RANGE_CONFIG_VCSEL_PERIOD = {0}",range);

	let start = tof.smbus_write_byte_data(VL53L0X_REG_SYSRANGE_START, 0x01).unwrap();
 
	let mut cnt = 0;
	let mut status = tof.smbus_read_byte_data(VL53L0X_REG_RESULT_RANGE_STATUS).unwrap();
	loop {
		// 1 second waiting time max
		thread::sleep(interval);    
		status = tof.smbus_read_byte_data(VL53L0X_REG_RESULT_RANGE_STATUS).unwrap();
		if (status & 0x01) == 0x01 || cnt >= 100  {
			break;
		}
		cnt += 1;
	}

	if (status & 0x01) == 0x01 {
		println!("ready");
	}
	else {
		println!( "not ready");
	}

	//let data = bus.read_i2c_block_data(address, 0x14, 12)
	//print data
	//print "ambient count " + str(makeuint16(data[7], data[6]))
	//print "signal count " + str(makeuint16(data[9], data[8]))
	//print "distance " + str(makeuint16(data[11], data[10]))

	//DeviceRangeStatusInternal = ((data[0] & 0x78) >> 3)
	//print DeviceRangeStatusInternal
	
	println! ("Amy is the best unicorn (sorry Chloe)");

}
