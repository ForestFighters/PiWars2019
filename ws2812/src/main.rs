use std::thread;
use std::time::Duration;

extern crate rs_ws281x;

use rs_ws281x::*;


fn left_on( controller: &mut Controller )
{
	// get the strand of LEDs on channel 1
	let leds = controller.leds_mut(0);
	
	leds[0] = [ 0, 0, 0, 0];
	leds[1] = [ 0, 0, 0, 0];
	leds[2] = [ 0, 0, 0, 0];
	leds[3] = [ 0, 0, 255, 0];
	leds[4] = [ 0, 0, 255, 0];
	leds[5] = [ 0, 0, 255, 0];
	
	println!("Left");
}

fn right_on( controller: &mut Controller )
{
	// get the strand of LEDs on channel 1
	let leds = controller.leds_mut(0);

	
	leds[0] = [ 0, 0, 255, 0];
	leds[1] = [ 0, 0, 255, 0];
	leds[2] = [ 0, 0, 255, 0];
	leds[3] = [ 0, 0, 0, 0];
	leds[4] = [ 0, 0, 0, 0];
	leds[5] = [ 0, 0, 0, 0];
	
	println!("Right");
		
}

fn all_on( controller: &mut Controller, brightness:  u8 )
{
	// get the strand of LEDs on channel 1
	let leds = controller.leds_mut(0);

	
	leds[0] = [ 0, 0, brightness, 0];
	leds[1] = [ 0, 0, brightness, 0];
	leds[2] = [ 0, 0, brightness, 0];
	leds[3] = [ 0, 0, brightness, 0];
	leds[4] = [ 0, 0, brightness, 0];
	leds[5] = [ 0, 0, brightness, 0];
	
	
	println!("{:?}", brightness);
		
}


fn main() {
	
	println!("Start");
	
	// Construct a single channel controller. Note that the
	// Controller is initialized by default and is cleaned up on drop
	let mut controller = ControllerBuilder::new()
		// default
		.freq(800_000)
		// default
		.dma(10)
		.channel( 0, 
			ChannelBuilder::new()
				.pin(12)
				.count(6)
				.strip_type(StripType::Ws2812)
				.brightness(50)
				.build()
		 )
		.build().unwrap();
			
										
    println!("Controller built");

	let mut check = 0;	
	let mut brightness = 0;	
	loop {
					
		if check == 0 {		
			left_on( &mut controller);
		}
		else {
			right_on( &mut controller);
		}
		
		//all_on( &mut controller, brightness);

		controller.render().unwrap();
		
		thread::sleep(Duration::from_millis(500));
		check = if check == 0 { 1 } else { 0 };
		brightness = if brightness >=20 { 0 } else { brightness + 1 };
	}
}

