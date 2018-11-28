extern crate rust_pigpio;
extern crate i2cdev;
extern crate byteorder;

use self::rust_pigpio::*;
use self::rust_pigpio::pwm::*;

pub struct Servo {
	pub pwm_pin: u32
}

impl Servo {	
	
	pub fn init( &self ){				
		set_mode(self.pwm_pin, OUTPUT).unwrap();		
		servo(self.pwm_pin, 0).unwrap();
		//set_pwm_frequency(self.pwm_pin, 500).unwrap();
		//set_pwm_range(self.pwm_pin, 1000).unwrap();
	}
	
	pub fn set_pulse_width( &self, mut width: u32 ) {
		if width < 500  {
			width = 500;
		}
		if width > 2500 {
			width = 2500;
		}
		servo(self.pwm_pin, width).unwrap();
	}	
		
}

pub fn build_servo( pwm_pin: u32 ) -> Servo {
	Servo {
		pwm_pin		
	}
}
