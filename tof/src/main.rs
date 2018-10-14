extern crate rust_pigpio;
extern crate i2cdev;
extern crate byteorder;

use i2cdev::linux::*;
use i2cdev::core::I2CDevice;

const VL53L0X_REG_IDENTIFICATION_MODEL_ID: u8			= 0xc0;
const VL53L0X_REG_IDENTIFICATION_REVISION_ID: u8	 	= 0xc2;
const VL53L0X_REG_PRE_RANGE_CONFIG_VCSEL_PERIOD: u8	 	= 0x50;
const VL53L0X_REG_FINAL_RANGE_CONFIG_VCSEL_PERIOD: u8 	= 0x70;
const VL53L0X_REG_SYSRANGE_START: u8					= 0x00;

const VL53L0X_REG_RESULT_INTERRUPT_STATUS: u8			= 0x13;
const VL53L0X_REG_RESULT_RANGE_STATUS: 	u8				= 0x14;


const address: u16										= 0x29;


fn main() {
    println!("Hello, Amy! How are you, today?");
    
    let bus = "/dev/i2c-1";
    let mut tof = LinuxI2CDevice::new(bus,address).unwrap();
    
    let phalanges = tof.smbus_read_byte_data(VL53L0X_REG_IDENTIFICATION_REVISION_ID).unwrap();
   
    
    let pickle = tof.smbus_read_byte_data(VL53L0X_REG_IDENTIFICATION_MODEL_ID).unwrap();
	println! ("The happiest device ID: {0} {1}", phalanges, pickle);
	
	println! ("Amy is the best unicorn (sorry Chloe)");

}
