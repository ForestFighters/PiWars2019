//extern crate hmc5883l;
//use hmc5883l::*;

extern crate i2cdev;

use i2cdev::linux::*;
use i2cdev::core::I2CDevice;

use std::thread;
use std::time::Duration;
use std::f32::consts::PI;

/// based on https://github.com/adafruit/Adafruit_HMC5883_Unified/blob/master/examples/magsensor/magsensor.ino

pub struct HMC5883L {
    compass: Box<LinuxI2CDevice>
}

impl HMC5883L {
	
	pub fn new(filename: &'static str, address: u16) -> Result<Self, Box<LinuxI2CError>> {

        let mut compass = try!(LinuxI2CDevice::new(filename, address));

        // set gain to +/1 1.3 Gauss
        try!(compass.smbus_write_byte_data(0x01, 0x20));

        // set in continuous-measurement mode
        try!(compass.smbus_write_byte_data(0x02, 0x00));

        // delay before taking first reading
        thread::sleep(Duration::from_millis(100));

        Ok(HMC5883L { compass: Box::new(compass) })
    }
    
    pub fn read(&mut self) -> Result<(f32, f32, f32), Box<LinuxI2CError>> {

        // read two bytes each from registers 03 through 05 (x, z, y)
        let mut buf: [u8; 6] = [0; 6];
        try!(self.compass.read(&mut buf));

        // start reading from register 03 (x value)
        try!(self.compass.smbus_write_byte(0x03));
        thread::sleep(Duration::from_millis(100));

        // parse the data in the correct order - x, z, y (NOT x, y, z as you would expect)
        let x : i16 = ((buf[0] as i16) << 8) as i16 | buf[1] as i16;
        let z : i16 = ((buf[2] as i16) << 8) as i16 | buf[3] as i16;
        let y : i16 = ((buf[4] as i16) << 8) as i16 | buf[5] as i16;

        // return tuple containing x, y, z values
        Ok((x as f32, y as f32, z as f32))
    }
    
    
    pub fn read_radians(&mut self) -> Result<(f32), Box<LinuxI2CError>> {
		
		let gauss_lsb_xy = 1100.0;
		let gauss_lsb_z  =  980.0;    
		let declination_angle = 0.00116355; // Suffolk in radians, not degrees
		
		// read raw values
        let (x, y, z) = self.read().unwrap();

        // convert to micro-teslas
        let (x, y, _z) = (x/gauss_lsb_xy*100.0, y/gauss_lsb_xy*100.0, z/gauss_lsb_z*100.0);

        let mut heading = y.atan2(x) + declination_angle;

        if heading < 0.0 {
            heading += 2.0 * PI;
        }

        if heading > 2.0 * PI {
            heading -= 2.0 * PI;
        }
        
        Ok(heading as f32)
        
	}
	
    
    pub fn read_degrees(&mut self) -> Result<(f32), Box<LinuxI2CError>> {
		
		let radians = self.read_radians().unwrap();
		
		let heading = radians * 180.0 / PI;
		 
        Ok(heading as f32)
    }
}


fn main() {
	
	const ADDRESS: u16	= 0x1E;
	
	let mut compass = HMC5883L::new( "/dev/i2c-1", ADDRESS).unwrap();
	
	loop {
		let heading = compass.read_degrees().unwrap();
		println!("heading={:.*}", 1, heading);
		thread::sleep(Duration::from_millis(500));
	}
	    
}
