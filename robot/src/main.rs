extern crate rust_pigpio;
extern crate i2cdev;
extern crate byteorder;

use std::thread::sleep;
use std::time::Duration;
use rust_pigpio::*;
use rust_pigpio::pwm::*;

use i2cdev::linux::*;
use i2cdev::core::*;

const PIN: u32 = 21;

const ADDR_ADXL345:			u16 = 0x53;

const EARTH_GRAVITY_MS2: 	f64 = 9.80665;
const SCALE_MULTIPLIER: 	f64 = 0.004;

const DATA_FORMAT: 			u8 = 0x31;
const BW_RATE: 				u8 = 0x2C;
const POWER_CTL: 			u8 = 0x2D;

const BW_RATE_1600HZ: 		u8 = 0x0F;
const BW_RATE_800HZ: 		u8 = 0x0E;
const BW_RATE_400HZ: 		u8 = 0x0D;
const BW_RATE_200HZ: 		u8 = 0x0C;
const BW_RATE_100HZ: 		u8 = 0x0B;
const BW_RATE_50HZ: 		u8 = 0x0A;
const BW_RATE_25HZ: 		u8 = 0x09;

const RANGE_2G: 			u8 = 0x00;
const RANGE_4G: 			u8 = 0x01;
const RANGE_8G: 			u8 = 0x02;
const RANGE_16G: 			u8 = 0x03;

const MEASURE: 				u8 = 0x08;
const AXES_DATA: 			u8 = 0x32;


fn main() {
    println!("Initialized pigpio. Version: {}", initialize().unwrap());
    set_mode(PIN, OUTPUT).unwrap();
    println!("Mode set!");
    write(PIN, OFF).unwrap();
    println!("Light off.");

    set_pwm_frequency(PIN, 500).unwrap(); // Set to modulate at 500hz.
    set_pwm_range(PIN, 1000).unwrap(); // Set range to 1000. 1 range = 2 us;
    
    sleep(Duration::from_secs(2));
    pwm(PIN, 1000).unwrap();
    println!("100%");

    sleep(Duration::from_secs(2));
    write(PIN, OFF).unwrap();
    println!("Light off.");
    
    let device = "/dev/i2c-1";
    let mut adxl345_i2cdev = LinuxI2CDevice::new(device, ADDR_ADXL345).unwrap();    
    // Bandwidth    
    adxl345_i2cdev.smbus_write_byte_data(BW_RATE, BW_RATE_100HZ);    
    
    // Range
    let rangedata = adxl345_i2cdev.smbus_read_byte_data(DATA_FORMAT);
    if rangedata.is_ok() {
		let mut value = rangedata.unwrap();
		let mut value = value & !0x0F;
        let mut value = value | RANGE_2G;
        let mut value = value | 0x08;
        				
		adxl345_i2cdev.smbus_write_byte_data(DATA_FORMAT, value);
		// Enable measurement
		adxl345_i2cdev.smbus_write_byte_data(POWER_CTL, MEASURE);
		
		let i2c_result = adxl345_i2cdev.smbus_read_block_data(AXES_DATA);
		println!("Result = {:?}",i2c_result.unwrap());
	}
	else {
		println!("Error = {:?}",rangedata.err());
	}
    
    terminate();
}
