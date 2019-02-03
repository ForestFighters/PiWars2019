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

//const VL53L0X_REG_RESULT_INTERRUPT_STATUS: u8         = 0x13;
const VL53L0X_REG_RESULT_RANGE_STATUS: u8 = 0x14;

const ADDRESS: u16 = 0x29;

pub struct VL53L0X {
    tof: Box<LinuxI2CDevice>,
}

impl VL53L0X {
    pub fn new(filename: &'static str) -> Result<Self, Box<LinuxI2CError>> {
        let mut tof = try!(LinuxI2CDevice::new(filename, ADDRESS));
        // delay before taking first reading
        thread::sleep(time::Duration::from_millis(100));

        let _revision = tof
            .smbus_read_byte_data(VL53L0X_REG_IDENTIFICATION_REVISION_ID)
            .unwrap();
        let _model = tof
            .smbus_read_byte_data(VL53L0X_REG_IDENTIFICATION_MODEL_ID)
            .unwrap();
        //println! ("Revision: {0} Model {1}", revision, model);

        let _prerange = tof
            .smbus_read_byte_data(VL53L0X_REG_PRE_RANGE_CONFIG_VCSEL_PERIOD)
            .unwrap();
        //println! ("PRE_RANGE_CONFIG_VCSEL_PERIOD = {0}",prerange);

        let _range = tof
            .smbus_read_byte_data(VL53L0X_REG_FINAL_RANGE_CONFIG_VCSEL_PERIOD)
            .unwrap();
        //println!("FINAL_RANGE_CONFIG_VCSEL_PERIOD = {0}",range);

        Ok(VL53L0X { tof: Box::new(tof) })
    }

    pub fn read(&mut self) -> Result<(u16), Box<LinuxI2CError>> {
        let interval = time::Duration::from_millis(1); // was 10
        let mut cnt = 0;
        let _start = self
            .tof
            .smbus_write_byte_data(VL53L0X_REG_SYSRANGE_START, 0x01);
        let mut status = self
            .tof
            .smbus_read_byte_data(VL53L0X_REG_RESULT_RANGE_STATUS)
            .unwrap();
        loop {
            if (status & 0x01) == 0x01 || cnt >= 100 {
                break;
            }
            // 1 second waiting time max
            thread::sleep(interval);
            status = self
                .tof
                .smbus_read_byte_data(VL53L0X_REG_RESULT_RANGE_STATUS)
                .unwrap();
            cnt += 1;
        }

        if (status & 0x01) != 0x01 {
            println!("not ready");
        }

        let data = self
            .tof
            .smbus_read_i2c_block_data(VL53L0X_REG_RESULT_RANGE_STATUS, 12)
            .unwrap();
        //println!("{:#?}",data);

        let dist1: u16 = (data[10]).into();
        let dist2: u16 = (data[11]).into();
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
