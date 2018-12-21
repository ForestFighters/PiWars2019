extern crate rust_pigpio;
extern crate i2cdev;
extern crate byteorder;
extern crate gilrs;
extern crate image;

extern crate robot;

use std::{thread, time};
use rust_pigpio::*;

use gilrs::{Gilrs, Button, Event };
use gilrs::Axis::{LeftZ, RightZ, RightStickX, RightStickY, LeftStickX, LeftStickY, DPadY, DPadX};

use robot::servo::*;
use robot::motor::*;
use robot::ssd1327::*;
use robot::hmc5883l::*;
use robot::vl53l0x::*;


fn _test() {
        
    // Test compass
    let mut compass = HMC5883L::new("/dev/i2c-9").unwrap();
    println!("Current Heading {:.*}", 1, compass.read_degrees().unwrap());
    
    // Test distance sensors
    let mut front = VL53L0X::new( "/dev/i2c-5").unwrap();
    let mut right = VL53L0X::new( "/dev/i2c-6").unwrap();
    let mut left = VL53L0X::new( "/dev/i2c-7").unwrap();
    let mut back = VL53L0X::new( "/dev/i2c-8").unwrap(); 
    
    println!("Front Distance {:.*}", 1, front.read().unwrap());
    println!("Right Distance {:.*}", 1, right.read().unwrap());
    println!("Left Distance {:.*}", 1, left.read().unwrap());
    println!("Back Distance {:.*}", 1, back.read().unwrap());
        
}

fn do_canyon( display: &mut SSD1327, gilrs: &mut Gilrs ) {
    
    while let Some(Event { id, event, time }) = gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event); 
            break;              
        }
        
    display.clear();   
}

fn do_hubble( display: &mut SSD1327, gilrs: &mut Gilrs ) {
    
    
    while let Some(Event { id, event, time }) = gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event); 
            break;              
        }
        
    display.clear();   
}


fn do_straight( display: &mut SSD1327, gilrs: &mut Gilrs ) {
       
	println!("Initialized pigpio. Version: {}", initialize().unwrap());
    let interval = time::Duration::from_millis(2000);

    //Use BCM numbering 
    // Channel 4
    let left_rear_motor = build_motor( 10, 11 ); 
    left_rear_motor.init();
        
    // Channel 3
    let right_rear_motor = build_motor( 9, 8 );
    right_rear_motor.init();
    
    // Channel 2
    let left_front_motor = build_motor( 15, 22 );
    left_front_motor.init();
    
    // Channel 1
    let right_front_motor = build_motor( 14, 27 );
    right_front_motor.init();    
    
    left_rear_motor.stop();    
    right_rear_motor.stop();
    left_front_motor.stop();
    right_front_motor.stop(); 
    
    let mut right = VL53L0X::new( "/dev/i2c-10").unwrap();
    let mut left = VL53L0X::new( "/dev/i2c-5").unwrap();
    let mut front = VL53L0X::new( "/dev/i2c-6").unwrap();
    
    display.clear(); 
    display.draw_text(4, 4, "Press start...", WHITE).unwrap();
    display.update_all().unwrap();
    
    let mut target: i32 = 0;
    
    let mut running = false;
    loop {
        while let Some(Event { id, event, time }) = gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event); 
            break;              
        }
        
        // Start button -> running
        if gilrs[0].is_pressed(Button::Start) {
            target = left.read().unwrap() as i32 - right.read().unwrap() as i32;
            display.draw_text(4, 4, "              ", WHITE).unwrap();
            display.update().unwrap();
            println!("Target {:?}", target); 
            running = true;
        } 
        
        
        // Triangle and cross to exit
        if gilrs[0].is_pressed(Button::West) && gilrs[0].is_pressed(Button::South) {
            break;
        } 
        
        
        if running {    
                           
			let mut left_rear_speed: i32;
			let mut right_rear_speed: i32;
			let mut left_front_speed: i32;
			let mut right_front_speed: i32;
			
            let right_dist: i32 = right.read().unwrap() as i32;
            let left_dist: i32 = left.read().unwrap() as i32;
            let front_dist: i32 = front.read().unwrap() as i32;
            
            println!("Front {:#?}mm, Right {:#?}mm, Left {:#?}mm ",front_dist, right_dist, left_dist);
            
            if front_dist < 400 {
				left_rear_motor.stop();    
				right_rear_motor.stop();
				left_front_motor.stop();
				right_front_motor.stop();  
				break;
			}
			            
            left_front_speed  = 1000;
            left_rear_speed   = 1000;
            right_front_speed = -1000;         
            right_rear_speed  = -1000;
            
            let difference: i32 = (target - (left_dist - right_dist)) * 2;
            
            if difference > 20 {
                // turn right
                println!("Turn Right {:04}  ", difference);
				left_front_speed  = left_front_speed;
				left_rear_speed   = left_rear_speed;
				right_front_speed = right_front_speed + difference;				
				right_rear_speed  = right_rear_speed + difference;      
                
            } else if difference < -20 {
                // turn left
                println!("Turn Left  {:04}  ", -difference);
                left_front_speed  = left_front_speed + difference;
                left_rear_speed   = left_rear_speed + difference;
				right_front_speed = right_front_speed;				
				right_rear_speed  = right_rear_speed;      
                
            } else {
                // straight
                //println!("Straight");                
            }
            
            //if left_rear_speed != 0 || right_rear_speed != 0 || left_front_speed != 0 || right_front_speed != 0  {
				//println!(" {0}, {1}, {2}, {3} ", left_rear_speed, right_rear_speed, left_front_speed, right_front_speed );
			//} 
            left_rear_motor.power(left_rear_speed);
			right_rear_motor.power(right_rear_speed);   
			left_front_motor.power(left_front_speed);
			right_front_motor.power(right_front_speed);                                 
        }
    }
    
    left_rear_motor.stop();    
    right_rear_motor.stop();
    left_front_motor.stop();
    right_front_motor.stop();   
    
    display.clear();  
    thread::sleep(interval);    
    terminate();  
}

fn do_wheels_rc( display: &mut SSD1327, gilrs: &mut Gilrs ) {
    
    println!("Initialized pigpio. Version: {}", initialize().unwrap());
    let interval = time::Duration::from_millis(2000);

    //Use BCM numbering 
    // Channel 4
    let left_rear_motor = build_motor( 10, 11 ); 
    left_rear_motor.init();
        
    // Channel 3
    let right_rear_motor = build_motor( 9, 8 );
    right_rear_motor.init();
    
    // Channel 2
    let left_front_motor = build_motor( 15, 22 );
    left_front_motor.init();
    
    // Channel 1
    let right_front_motor = build_motor( 14, 27 );
    right_front_motor.init();    
    
    left_rear_motor.stop();    
    right_rear_motor.stop();
    left_front_motor.stop();
    right_front_motor.stop(); 
    
    let servo = build_servo( 21 );
    
    let mut gear = 1;
    
    
    loop {
        while let Some(Event { id, event, time }) = gilrs.next_event() {
                println!("{:?} New event from {}: {:?}", time, id, event); 
                break;              
            }
                    
        let mut left_stick_y = 0;
        let mut right_stick_y = 0;        
        
        let mut dpad = 0;
                
        if gilrs[0].axis_data(LeftStickY).is_some() {               
            left_stick_y = (gilrs[0].axis_data(LeftStickY).unwrap().value() * 1000.0) as i32;
        }
        
        // PS4 controller
        if gilrs[0].axis_data(RightStickY).is_some() {               
            right_stick_y = (gilrs[0].axis_data(RightStickY).unwrap().value() * 1000.0) as i32;
        }
        if gilrs[0].is_pressed(Button::DPadUp) {
            dpad = 1;
        }        
        if gilrs[0].is_pressed(Button::DPadDown) {
            dpad = -1;
        }
       
       
        // PiHut controller  
        if gilrs[0].axis_data(RightZ).is_some() {               
            right_stick_y = (gilrs[0].axis_data(RightZ).unwrap().value() * -1000.0) as i32;
        }        
        if gilrs[0].axis_data(DPadY).is_some() {                
            dpad = (gilrs[0].axis_data(DPadY).unwrap().value()) as i32;
        }   
        
        
                    
        // Gear changing
        if gilrs[0].is_pressed(Button::North) {
            gear = 1;
            display.draw_text(4, 4, &gear.to_string(), LT_GREY).unwrap();
            display.update().unwrap();  
            println!(" {0} ",gear);         
        }
        
        if gilrs[0].is_pressed(Button::West) {
            gear = 2;
            display.draw_text(4, 4, &gear.to_string(), LT_GREY).unwrap();
            display.update().unwrap();  
            println!(" {0} ",gear);         
        }
        
        if gilrs[0].is_pressed(Button::East) {
            gear = 3;           
            display.draw_text(4, 4, &gear.to_string(), LT_GREY).unwrap();
            display.update().unwrap();  
            println!(" {0} ",gear);
        }
        
        if gilrs[0].is_pressed(Button::South) {
            gear = 4;           
            display.draw_text(4, 4, &gear.to_string(), LT_GREY).unwrap();
            display.update().unwrap();  
            println!(" {0} ",gear);
        }
            
        // Triangle and cross to exit
        if gilrs[0].is_pressed(Button::West) && gilrs[0].is_pressed(Button::South) {
            break;
        }  
        
        if dpad == 1 {
            servo.set_pulse_width( 2500 );
        }
        else if dpad == -1 {
            servo.set_pulse_width( 500 );
        }
        
        let mut left_rear_speed: i32;
        let mut right_rear_speed: i32;
        let mut left_front_speed: i32;
        let mut right_front_speed: i32;
                           
        if left_stick_y == 0 && right_stick_y == 0 {                
            left_rear_speed = 0;
            right_rear_speed = 0;
            left_front_speed = 0;
            right_front_speed = 0;
        }
        else
        {
            left_front_speed  = left_stick_y;
            left_rear_speed   = left_stick_y;
            right_front_speed = -right_stick_y;         
            right_rear_speed  = -right_stick_y;
        }
        
        left_front_speed  = left_front_speed / gear;
        right_front_speed = right_front_speed / gear;
        left_rear_speed   = left_rear_speed / gear;
        right_rear_speed  = right_rear_speed / gear;        
        
        //if left_rear_speed != 0 || right_rear_speed != 0 || left_front_speed != 0 || right_front_speed != 0  {
            //println!(" {0}, {1}, {2}, {3} ", left_rear_speed, right_rear_speed, left_front_speed, right_front_speed );
        //}   
        
        left_rear_motor.power(left_rear_speed);
        right_rear_motor.power(right_rear_speed);   
        left_front_motor.power(left_front_speed);
        right_front_motor.power(right_front_speed);     
            
    } 
    left_rear_motor.stop();    
    right_rear_motor.stop();
    left_front_motor.stop();
    right_front_motor.stop();   
    
    display.clear();  
    thread::sleep(interval);    
    terminate();
}

fn do_mecanum_rc( display: &mut SSD1327, gilrs: &mut Gilrs ) {
    
    println!("Initialized pigpio. Version: {}", initialize().unwrap());
    let interval = time::Duration::from_millis(2000);

    //Use BCM numbering
    let left_rear_motor = build_motor( 17, 27);
    left_rear_motor.init();
        
    let right_rear_motor = build_motor( 25, 23);
    right_rear_motor.init();
    
    let left_front_motor = build_motor( 10, 11);
    left_front_motor.init();
    
    let right_front_motor = build_motor( 12, 8);
    right_front_motor.init();
    
    let servo = build_servo( 21 );        
        
    let mut gear = 1;
    
    loop {
        while let Some(Event { id, event, time }) = gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event); 
            break;              
        }
                    
        let mut left_stick_x = 0;
        let mut left_stick_y = 0;
        let mut right_stick_y = 0;
        let mut right_stick_x = 0;
        
        let mut dpad = 0;
                
        if gilrs[0].axis_data(LeftStickY).is_some() {               
            left_stick_y = (gilrs[0].axis_data(LeftStickY).unwrap().value() * 1000.0) as i32;
        }
        
        if gilrs[0].axis_data(LeftStickX).is_some() {               
            left_stick_x = (gilrs[0].axis_data(LeftStickX).unwrap().value() * 1000.0) as i32;
        }
        
        if gilrs[0].axis_data(RightZ).is_some() {               
            right_stick_y = (gilrs[0].axis_data(RightZ).unwrap().value() * -1000.0) as i32;
        }
        
        if gilrs[0].axis_data(LeftZ).is_some() {                
            right_stick_x = (gilrs[0].axis_data(LeftZ).unwrap().value() * 1000.0) as i32;   
        }   
        
        if gilrs[0].axis_data(DPadY).is_some() {                
            dpad = (gilrs[0].axis_data(DPadY).unwrap().value()) as i32;
        }   
                
        
        if gilrs[0].is_pressed(Button::North) {
            gear = 1;
            display.draw_text(4, 4, &gear.to_string(), LT_GREY).unwrap();
            display.update().unwrap();  
            println!(" {0} ",gear);         
        }
        
        if gilrs[0].is_pressed(Button::West) {
            gear = 2;
            display.draw_text(4, 4, &gear.to_string(), LT_GREY).unwrap();
            display.update().unwrap();  
            println!(" {0} ",gear);         
        }
        
        if gilrs[0].is_pressed(Button::East) {
            gear = 3;           
            display.draw_text(4, 4, &gear.to_string(), LT_GREY).unwrap();
            display.update().unwrap();  
            println!(" {0} ",gear);
        }
        
        if gilrs[0].is_pressed(Button::South) {
            gear = 4;           display.clear(); 
            display.draw_text(4, 4, "Canyon...", LT_GREY).unwrap();
            display.update_all().unwrap();
            display.draw_text(4, 4, &gear.to_string(), LT_GREY).unwrap();
            display.update().unwrap();  
            println!(" {0} ",gear);
        }
                    
        if gilrs[0].is_pressed(Button::West) && gilrs[0].is_pressed(Button::South) {
            break;
        }
                
        
        let mut left_rear_speed: i32;
        let mut right_rear_speed: i32;
        let mut left_front_speed: i32;
        let mut right_front_speed: i32;
                
        if left_stick_y == 0 && left_stick_x == 0 {             
            left_rear_speed = 0;
            right_rear_speed = 0;
            left_front_speed = 0;
            right_front_speed = 0;
        }
        else
        {           
            left_front_speed  = -left_stick_x + left_stick_y - right_stick_x;               
            left_rear_speed   = left_stick_x + left_stick_y - right_stick_x;
            
            right_rear_speed  = -left_stick_x - left_stick_y + right_stick_x;
            right_front_speed = left_stick_x - left_stick_y + right_stick_x;
        }
        
        if dpad == 1 {
            servo.set_pulse_width( 2500 );
        }
        else if dpad == -1 {
            servo.set_pulse_width( 500 );
        }
        
        left_front_speed  = left_front_speed / gear;
        right_front_speed = right_front_speed / gear;
        left_rear_speed   = left_rear_speed / gear;
        right_rear_speed  = right_rear_speed / gear;        
        
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
    terminate();  
}

fn show_menu( display: &mut SSD1327, menu: i8) {display.clear();
    display.draw_text(20, 42, "Forest", WHITE).unwrap();
    display.draw_text(20, 50, "Fighters", WHITE).unwrap();
    display.draw_text(20, 58, "Ready...", WHITE).unwrap();
    display.update_all().unwrap();  
    
    display.clear(); 
    display.draw_text(4, 4, "Forest Fighters", LT_GREY).unwrap();
    
    if menu == 0 {
        let tiny = image::open("The Canyons of Mars Menu Item.jpg").unwrap();

        display.draw_image( 0, 16, tiny ).unwrap();
        display.draw_text(4, 108, "Canyons of Mars", WHITE).unwrap();
    }
    else if menu == 1 {
        let tiny = image::open("Hubble Telescope Item Menu.jpg").unwrap();

        display.draw_image( 0, 16, tiny ).unwrap();
        display.draw_text(12, 108, "Hubble T'scope", WHITE).unwrap();
    }
    else if menu == 2 {
        let tiny = image::open("Blast Off Menu Item.jpg").unwrap();

        display.draw_image( 0, 16, tiny ).unwrap();
        display.draw_text(40, 108, "Blast Off", WHITE).unwrap();
    }
    else if menu == 3 {
        let tiny = image::open("Large Wheels Menu Item.jpg").unwrap();

        display.draw_image( 0, 16, tiny ).unwrap();
        display.draw_text(4, 108, "Large Wheels RC", WHITE).unwrap();
    }
    else if menu == 4 {
        let tiny = image::open("Mecanum Wheels Menu Item.jpg").unwrap();

        display.draw_image( 0, 16, tiny ).unwrap();
        display.draw_text(28, 108, "Mecanum RC", WHITE).unwrap();
    }
    else if menu == 5 {
        let tiny = image::open("Exit Menu Item.jpg").unwrap();

        display.draw_image( 0, 16, tiny ).unwrap();
        display.draw_text(56, 108, "EXIT", WHITE).unwrap();
    }
    else if menu == 6 {
        let tiny = image::open("Shutdown Menu Item.jpg").unwrap();

        display.draw_image( 0, 16, tiny ).unwrap();
        display.draw_text(32, 108, "SHUTDOWN", WHITE).unwrap();
    }
         
    display.update_all().unwrap();   
    
}

fn main() {
          
    let mut display = SSD1327::new("/dev/i2c-3");
    display.begin().unwrap(); 
    
    display.clear();
    display.draw_text(20, 42, "Forest", WHITE).unwrap();
    display.draw_text(20, 50, "Fighters", WHITE).unwrap();
    display.draw_text(20, 58, "Ready...", WHITE).unwrap();
    display.update_all().unwrap();   
    
    let mut gilrs = Gilrs::new().unwrap();
    
    let mut menu :i8 = 0;      
    let mut prev :i8 = -1;
    
    loop {
        
        if menu > 6 {
            menu = 0;
        }
        else if menu < 0 {
            menu = 6;
        }
        
        if menu != prev {
            prev = menu;
            show_menu( &mut display, menu );     
        }
        
        while let Some(Event { id, event, time }) = gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event); 
            break;              
        }
        
        if ( gilrs[0].axis_data(DPadX).is_some() && gilrs[0].axis_data(DPadX).unwrap().value() == 1.0 ) || gilrs[0].is_pressed(Button::DPadRight) {                
            menu = menu + 1;
        }  
        
        if ( gilrs[0].axis_data(DPadX).is_some() && gilrs[0].axis_data(DPadX).unwrap().value() == -1.0 ) || gilrs[0].is_pressed(Button::DPadLeft) {                
            menu = menu - 1;
        }
        
        if gilrs[0].is_pressed(Button::Select) && menu == 0 {
            display.clear(); 
            display.draw_text(4, 4, "Canyon...", LT_GREY).unwrap();
            display.update_all().unwrap();  
            do_canyon( &mut display, &mut gilrs );
            prev = -1;
        }     
        
        if gilrs[0].is_pressed(Button::Select) && menu == 1 {
            display.clear(); 
            display.draw_text(4, 4, "Hubble...", LT_GREY).unwrap();
            display.update_all().unwrap();  
            do_hubble( &mut display, &mut gilrs );
            prev = -1;
        }   
        
        if gilrs[0].is_pressed(Button::Select) && menu == 2 {
            display.clear(); 
            display.draw_text(4, 4, "Blast Off...", LT_GREY).unwrap();
            display.update_all().unwrap();  
            do_straight( &mut display, &mut gilrs );
            prev = -1;
        }
        
        if gilrs[0].is_pressed(Button::Select) && menu == 3 {
            display.clear(); 
            display.draw_text(4, 4, "Wheels RC...", LT_GREY).unwrap();
            display.update_all().unwrap();  
            do_wheels_rc( &mut display, &mut gilrs );
            prev = -1;
        }    
        
        if gilrs[0].is_pressed(Button::Select) && menu == 4 {
            display.clear(); 
            display.draw_text(4, 4, "Mecanum RC...", LT_GREY).unwrap();
            display.update_all().unwrap();  
            do_mecanum_rc( &mut display, &mut gilrs );
            prev = -1;
        }    
        
        if gilrs[0].is_pressed(Button::Select) && gilrs[0].is_pressed(Button::West) && menu == 5 {
            display.clear(); 
            display.draw_text(4, 4, "Exiting...", LT_GREY).unwrap();
            display.update_all().unwrap();  
            break;
        }
        
        if gilrs[0].is_pressed(Button::Select) && gilrs[0].is_pressed(Button::West) && menu == 6 {
            display.clear(); 
            display.draw_text(4, 4, "Shutdown...", LT_GREY).unwrap();
            display.update_all().unwrap();  
            break;
        }
                
    }
    
    thread::sleep(time::Duration::from_millis(2000));
    display.clear(); 
    display.update_all().unwrap();  
    
}
