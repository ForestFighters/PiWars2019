extern crate wiringpi;
extern crate gilrs;

use wiringpi::pin::Value::{High, Low};
use wiringpi::pin::{SoftPwmPin, OutputPin};
use std::{thread, time};

use gilrs::{Gilrs, Button, Event };
use gilrs::Axis::{LeftZ, RightZ, LeftStickX, LeftStickY};

struct Motor {			
	pwm_pin: SoftPwmPin<wiringpi::pin::Phys>,
	fwd_pin: OutputPin<wiringpi::pin::Phys>,
	bwd_pin: OutputPin<wiringpi::pin::Phys>,
	arrow_pin: OutputPin<wiringpi::pin::Phys>
}
	
impl Motor {	
	
	fn init( &self ){				
		self.fwd_pin.digital_write(High);
		self.bwd_pin.digital_write(Low);		
		self.pwm_pin.pwm_write(0);	
		self.arrow_pin.digital_write(Low);	
	}
	
	fn power( &self, power: i32 ) {
		if power > 0 {
			self.forward( power );
		}
		else {
			self.backward( power * -1 );
		}
	}

	fn forward( &self, power: i32 ) {		
		self.fwd_pin.digital_write(High);
		self.bwd_pin.digital_write(Low);
		self.pwm_pin.pwm_write(power);
		self.arrow_pin.digital_write(High);	
	}
	
	fn backward( &self, power: i32 ) {		
		self.fwd_pin.digital_write(Low);
		self.bwd_pin.digital_write(High);
		self.pwm_pin.pwm_write(power);
		self.arrow_pin.digital_write(Low);	
	}
	
	fn stop( &self ){				
		self.fwd_pin.digital_write(High);
		self.bwd_pin.digital_write(Low);		
		self.pwm_pin.pwm_write(0);
		self.arrow_pin.digital_write(Low);	
	}

}

fn build_motor( pwm_pin: SoftPwmPin<wiringpi::pin::Phys>, fwd_pin: OutputPin<wiringpi::pin::Phys>, 	bwd_pin: OutputPin<wiringpi::pin::Phys>, arrow_pin: OutputPin<wiringpi::pin::Phys> ) -> Motor {
	Motor {
		pwm_pin,
		fwd_pin,
		bwd_pin,
		arrow_pin
	}
}



//motorpins = 
			 //"right_front_motor":{"config":{1:{"e":32,"f":24,"r":26},2:{"e":32,"f":26,"r":24}},"arrow":1},
			 //"left_front_motor":{"config":{1:{"e":19,"f":21,"r":23},2:{"e":19,"f":23,"r":21}}, "arrow":2},
			 //"right_rear_motor":{"config":{1:{"e":22,"f":16,"r":18},2:{"e":22,"f":18,"r":16}}, "arrow":3},
			 //"left_rear_motor":{"config":{1:{"e":11,"f":15,"r":13},2:{"e":11,"f":13,"r":15}},"arrow":4}}			
			 // arrowpins={1:33,2:35,3:37,4:36} 
                 
fn main() {
    //Setup WiringPi with its own pin numbering order
    let pi = wiringpi::setup_phys();
    let interval = time::Duration::from_millis(2000);

    //Use WiringPi  output    
    let left_rear_motor = build_motor( pi.soft_pwm_pin(11), pi.output_pin(15), pi.output_pin(13), pi.output_pin(33) );       
	left_rear_motor.init();
		
	let right_rear_motor = build_motor( pi.soft_pwm_pin(22), pi.output_pin(16), pi.output_pin(18), pi.output_pin(35) );       	
	right_rear_motor.init();
	
	let left_front_motor = build_motor( pi.soft_pwm_pin(19), pi.output_pin(21), pi.output_pin(23), pi.output_pin(37) );		
	left_front_motor.init();
	
	let right_front_motor = build_motor( pi.soft_pwm_pin(32), pi.output_pin(24), pi.output_pin(26), pi.output_pin(36) );	
	right_front_motor.init();
	    
    let mut gilrs = Gilrs::new().unwrap();

	// Iterate over all connected gamepads
	for (_id, gamepad) in gilrs.gamepads() {
		println!("{} is {:?}", gamepad.name(), gamepad.power_info());
	}

	loop {
		while let Some(Event { id, event, time }) = gilrs.next_event() {
			println!("{:?} New event from {}: {:?}", time, id, event); 	
		}
					
		let mut left_stick_x = 0;
		let mut left_stick_y = 0;
		let mut right_stick_y = 0;
		let mut right_stick_x = 0;
				
		if gilrs[0].axis_data(LeftStickY).is_some() {				
			left_stick_y = (gilrs[0].axis_data(LeftStickY).unwrap().value() * 400.0) as i32;
		}
		
		if gilrs[0].axis_data(LeftStickX).is_some() {				
			left_stick_x = (gilrs[0].axis_data(LeftStickX).unwrap().value() * 400.0) as i32;
		}
		
		if gilrs[0].axis_data(RightZ).is_some() {				
			right_stick_y = (gilrs[0].axis_data(RightZ).unwrap().value() * -400.0) as i32;
		}
		
		
		if gilrs[0].axis_data(LeftZ).is_some() {				
			right_stick_x = (gilrs[0].axis_data(LeftZ).unwrap().value() * 400.0) as i32;	
		}		
		
				
		if gilrs[0].is_pressed(Button::West) && gilrs[0].is_pressed(Button::South) {
			break;
		}
		
		//if left_stick_Y != 0 || bwd != 0 || left != 0 || right != 0  {
			//println!(" {0}, {1}, {2}, {3} ", left_stick_Y, bwd, left, right );
		//}		
		
		let left_rear_speed: i32;
		let right_rear_speed: i32;
		let left_front_speed: i32;
		let right_front_speed: i32;
				
		
		if left_stick_y == 0 && left_stick_x == 0 {				
			left_rear_speed = 0;
			right_rear_speed = 0;
			left_front_speed = 0;
			right_front_speed = 0;
		}
		else
		{			
			left_front_speed  = left_stick_x + left_stick_y + right_stick_x;
			right_front_speed = -left_stick_x + left_stick_y - right_stick_x;
			left_rear_speed   = -left_stick_x + left_stick_y + right_stick_x;
			right_rear_speed  = left_stick_x + left_stick_y - right_stick_x;
		}
		
		
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
}

