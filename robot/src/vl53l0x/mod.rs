// ---------------------------------- 8< ---------------------------------------------------------
extern crate byteorder;
extern crate i2cdev;

use self::i2cdev::core::I2CDevice;
use self::i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use std::{thread, time};

const VL53L0X_REG_IDENTIFICATION_MODEL_ID: u8 = 0xc0;
const VL53L0X_REG_IDENTIFICATION_REVISION_ID: u8 = 0xc2;
const VL53L0X_REG_PRE_RANGE_CONFIG_VCSEL_PERIOD: u8 = 0x50;
const VL53L0X_REG_FINAL_RANGE_CONFIG_VCSEL_PERIOD: u8 = 0x70;
const VL53L0X_REG_SYSRANGE_START: u8 = 0x00;
const VL53L0X_REG_SYSRANGE_MODE_BACKTOBACK: u8 = 0x02;

//const VL53L0X_REG_RESULT_INTERRUPT_STATUS: u8 = 0x13;
const VL53L0X_REG_RESULT_RANGE_STATUS: u8 = 0x14;

const ADDRESS: u16 = 0x29;

pub struct VL53L0X {
    tof: Box<LinuxI2CDevice>,
    stop_variable: u8,
    last_read: u16,
}

impl VL53L0X {
    pub fn new(filename: &'static str) -> Result<Self, Box<LinuxI2CError>> {
        let mut tof = try!(LinuxI2CDevice::new(filename, ADDRESS));
        // delay before taking first reading
        thread::sleep(time::Duration::from_millis(100));

        let _revision = try!(tof.smbus_read_byte_data(VL53L0X_REG_IDENTIFICATION_REVISION_ID));

        let _model = try!(tof.smbus_read_byte_data(VL53L0X_REG_IDENTIFICATION_MODEL_ID));

        let _prerange = try!(tof.smbus_read_byte_data(VL53L0X_REG_PRE_RANGE_CONFIG_VCSEL_PERIOD));

        let _range = try!(tof.smbus_read_byte_data(VL53L0X_REG_FINAL_RANGE_CONFIG_VCSEL_PERIOD));
                
        let sv = try!(tof.smbus_read_byte_data(0x91));
        //println! ("Revision: {0} Model {1}", revision, model);
        //println! ("PRE_RANGE_CONFIG_VCSEL_PERIOD = {0}",prerange);
        //println!("FINAL_RANGE_CONFIG_VCSEL_PERIOD = {0}",range);

        Ok(VL53L0X { tof: Box::new(tof), stop_variable: sv, last_read: 0 })
    }
    
     pub fn start_continuous(&mut self) -> Result<(), Box<LinuxI2CError>> {
        try!(self.tof.smbus_write_byte_data(0x80, 0x01));
        try!(self.tof.smbus_write_byte_data(0xFF, 0x01));
        try!(self.tof.smbus_write_byte_data(0x00, 0x00));
        let sv = self.stop_variable;
        try!(self.tof.smbus_write_byte_data(0x91, sv));
        try!(self.tof.smbus_write_byte_data(0x00, 0x01));
        try!(self.tof.smbus_write_byte_data(0xFF, 0x00));
        try!(self.tof.smbus_write_byte_data(0x80, 0x00));

        //continuous back-to-back mode
        //VL53L0X_REG_SYSRANGE_MODE_BACKTOBACK        
        try!(self.tof.smbus_write_byte_data(VL53L0X_REG_SYSRANGE_START, VL53L0X_REG_SYSRANGE_MODE_BACKTOBACK));

        Ok(())
    }
    
    pub fn read_continous( &mut self ) -> u16 {
        
        let dist = match self.read_continous_internal() {
            Ok(dist) => dist,
            Err(e) => {
                println!("Bad read {:?}", e);
                return self.last_read;
            }
        };                   
        self.last_read = dist;                
        return dist;  
    }
    
    pub fn read(&mut self) -> u16 {        
        
        let dist = match self.read_internal() {
            Ok(dist) => dist,
            Err(e) => {
                println!("Bad read {:?}", e);
                return self.last_read;
            }
        };   
        self.last_read = dist;                     
        return dist;  
    }
    
    fn read_continous_internal(&mut self) -> Result<(u16), Box<LinuxI2CError>> {
        
        let data = try!(self.tof.smbus_read_i2c_block_data(VL53L0X_REG_RESULT_RANGE_STATUS, 12));        
        let dist1: u16 = (data[10]).into();
        let dist2: u16 = (data[11]).into();
        let mut distance = (dist1 * 256) + dist2;
        if distance <= 20  {
            distance = 9999;
        }
        if distance > 1900 {
            distance = 9999;
        }
        Ok(distance as u16)
    }

    fn read_internal(&mut self) -> Result<(u16), Box<LinuxI2CError>> {
        let interval = time::Duration::from_millis(2); // was 10
        let mut cnt = 0;
        let _start = try!(self.tof.smbus_write_byte_data(VL53L0X_REG_SYSRANGE_START, 0x01));
        let mut status = try!(self.tof.smbus_read_byte_data(VL53L0X_REG_RESULT_RANGE_STATUS));
        loop {
            if (status & 0x01) == 0x01 || cnt >= 1000 {
                break;
            }
            // 1 second waiting time max
            thread::sleep(interval);
            status = try!(self.tof.smbus_read_byte_data(VL53L0X_REG_RESULT_RANGE_STATUS));
            cnt += 1;
        }

        if (status & 0x01) != 0x01 {
            println!("not ready");
        }

        let data = try!(self.tof.smbus_read_i2c_block_data(VL53L0X_REG_RESULT_RANGE_STATUS, 12));        

        let dist1: u16 = (data[10]).into();
        let dist2: u16 = (data[11]).into();
        let mut distance = (dist1 * 256) + dist2;
        if distance <= 20 || distance > 1900 {
            distance = 9999
        }
        Ok(distance as u16)
    }
}
// ---------------------------------- 8< ---------------------------------------------------------
