extern crate byteorder;
extern crate csv;
extern crate gilrs;
extern crate i2cdev;
extern crate image;
extern crate rust_pigpio;
extern crate serde;
extern crate serde_json;

extern crate robot;

//use rust_pigpio::*;
use std::fs;
use std::process::Command;
use std::str;
use std::sync::mpsc::{self, TryRecvError};
use std::time::Duration;
use std::time::Instant;
use std::{thread, time};

use gilrs::Axis::{DPadX, DPadY, LeftStickX, LeftStickY, LeftZ, RightStickX, RightStickY, RightZ};
use gilrs::{Button, Event, EventType, Gilrs};

use serde::{Deserialize, Serialize};
use serde_json::Result;

use robot::camera::*;
use robot::context::*;
use robot::control::*;
use robot::hmc5883l::*;
use robot::motor::*;
use robot::pixel::*;
use robot::servo::*;
use robot::ssd1327::*;
use robot::vl53l0x::*;

#[derive(PartialEq)]
enum Rotation {
    StartLeft,
    StartRight,
}

#[derive(PartialEq)]
enum Activities {
    Waiting,
    Searching,
    MoveTowards,
    MoveAway,
    Complete,
    Done,
    Finished,
    Test,
}

const NONE: i32 = -1;
const RED: i32 = 0;
const BLUE: i32 = 1;
const YELLOW: i32 = 2;
const GREEN: i32 = 3;
const PURPLE: i32 = 4;
const CYAN: i32 = 5;
const ALL: i32 = 6;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calibrate {
    pub red_lower: [f64; 4],
    pub red_upper: [f64; 4],

    pub green_lower: [f64; 4],
    pub green_upper: [f64; 4],

    pub blue_lower: [f64; 4],
    pub blue_upper: [f64; 4],

    pub yellow_lower: [f64; 4],
    pub yellow_upper: [f64; 4],
}

fn _test() {
    //// Test compass
    //let mut compass = HMC5883L::new("/dev/i2c-1").unwrap();
    //println!("Compass started");

    // Test distance sensors
    let mut leftfront = VL53L0X::new("/dev/i2c-5").unwrap();
    println!("left front started");
    let mut leftback = VL53L0X::new("/dev/i2c-6").unwrap();
    println!("left back started");
    let mut back = VL53L0X::new("/dev/i2c-7").unwrap();
    println!("back started");
    let mut front = VL53L0X::new("/dev/i2c-8").unwrap();
    println!("front started");
    let mut rightfront = VL53L0X::new("/dev/i2c-10").unwrap();
    println!("right front started");
    let mut rightback = VL53L0X::new("/dev/i2c-9").unwrap();
    println!("right back started");

    loop {
        //println!(
            //"\x1B[HCurrent Heading {:.*}  ",
            //1,
            //compass.read_degrees().unwrap()
        //);
        println!("Left Back Distance   {:.*}   ", 1, leftback.read());
        println!("Left Front Distance  {:.*}   ", 1, leftfront.read());
        println!("Back Distance        {:.*}   ", 1, back.read());
        println!("Front Distance       {:.*}   ", 1, front.read());
        println!("Right Back Distance  {:.*}   ", 1, rightback.read());
        println!("Right Front Distance {:.*}   ", 1, rightfront.read());
    }
}

fn _test2() {
    let mut cam = build_camera();

    loop {
        let colour = cam.get_colour(true);
        //print_colour(colour);
    }
}

fn _test3() {
    let mut pixel = build_pixel();
    loop {
        pixel.red();
        pixel.render();
        println!("Red");
        thread::sleep(time::Duration::from_millis(1000));
        pixel.green();
        pixel.render();
        println!("Green");
        thread::sleep(time::Duration::from_millis(1000));
        pixel.blue();
        pixel.render();
        println!("Blue");
        thread::sleep(time::Duration::from_millis(1000));
        pixel.yellow();
        pixel.render();
        println!("Yellow");
        thread::sleep(time::Duration::from_millis(1000));
    }
}

fn _test4() {
    let mut display = SSD1327::new("/dev/i2c-3");
    display.begin().unwrap();
    display.clear();
    display.draw_text(4, 4, "Canyon...", LT_GREY).unwrap();
    let tiny = image::open("The Canyons of Mars Menu Item.jpg").unwrap();
    display.draw_image(0, 16, tiny).unwrap();
    display.update_all().unwrap();
    let mut pixel = build_pixel();
    pixel.red();
    pixel.render();
    println!("Red");
}

fn _test5() {
    let mut control = build_control();
    control.init();
    control.gear = 1;
    control.speed(800, 800, 800, 800);
    thread::sleep(time::Duration::from_millis(5000));
    control.gear = 2;
    control.speed(800, 800, 800, 800);
    thread::sleep(time::Duration::from_millis(5000));
    control.gear = 3;
    control.speed(800, 800, 800, 800);
    thread::sleep(time::Duration::from_millis(5000));
    control.gear = 4;
    control.speed(800, 800, 800, 800);
    thread::sleep(time::Duration::from_millis(5000));
    control.stop();
}


const MINDIST: u16 = 300;
const MAXDIST: u16 = 500;
const SPEED: i32 = 350;
fn get_deceleration(distance: u16, min: u16) -> f64 {
    
    if distance < min {
        return 0.0;
    }
    let distance_togo = distance - min;
    let mut decel = 1.0;
    if distance_togo < min {
        decel = ((distance_togo as f64) / min as f64);
        if decel < 0.4 {
            decel = 0.4;
        }
    }
    //println!("decel is: {:?}", decel);
    return decel;
}

//fn _calc_target(original: f32, heading: f32) -> f32 {
    //let mut target = 0.0;
    //if heading > original {
        //// heading 270 > original 5
        //// Is the distance between (360 - heading + original) (A)
        //// 360 - 270 + 5 = 95
        //let a = 360.0 - heading + original;
        //// Greater than distance between  heading - original (B)
        //// 270 - 5 = 265
        //let b = heading - original;
        //if a < b {
            //target = a;
        //} else {
            //target = -b;
        //}
    //} else {
        //// heading 5 < original 270
        //// Is the distance between (360 - original + heading) (A)
        //// 360 - 270 + 5 = 95
        //let a = 360.0 - heading + original;
        //// Greater than distance between  original - heading (B)
        //// 270 - 5 = 265
        //let b = heading - original;
        //if a < b {
            //target = -a;
        //} else {
            //target = b;
        //}
    //}

    //return target;
//}


//fn _align(original: f32, compass: &mut HMC5883L, control: &mut Control, cam: &mut Camera, gear: i32) {
    //control.stop();
    //let mut heading = compass.read_degrees().unwrap();
    //let mut diff = calc_target(original, heading);

    //if diff < 0.5 && diff > -0.5 {
        //return;
    //}

    //if diff > 0.0 {
        //while diff > 1.0 {
            //heading = compass.read_degrees().unwrap();
            //diff = calc_target(original, heading);
            //println!(
                //"Original {:#?}°  Current {:#?}°  Diff {:#?}",
                //original, heading, diff
            //);
            //control.turn_left(SPEED, gear);
            //cam.discard_video();
        //}
    //} else {
        //while diff < -1.0 {
            //heading = compass.read_degrees().unwrap();
            //diff = calc_target(original, heading);
            //println!(
                //"Original {:#?}°  Current {:#?}°  Diff {:#?}",
                //original, heading, diff
            //);
            //control.turn_right(SPEED, gear);
            //cam.discard_video();
        //}
    //}
    //control.stop();
//}


fn do_canyon(context: &mut Context) {
    
    let interval = time::Duration::from_millis(50);
    
    //let mut compass = HMC5883L::new("/dev/i2c-1").unwrap();
    
    // Distance sensors
    let mut front = try_open_tof("/dev/i2c-8");
    let mut leftfront = try_open_tof("/dev/i2c-5");
    let mut rightfront = try_open_tof("/dev/i2c-10");
    let mut back = try_open_tof("/dev/i2c-7");
    println!("front started");
    println!("left front started");
    println!("right front started");
    println!("back started");

    set_continous(&mut front);
    set_continous(&mut leftfront);
    set_continous(&mut rightfront);
    set_continous(&mut back);

    let mut control = build_control();
    control.init();

    let mut distance: u16 = 0;
    let mut direction = "Forward";
    let mut prev_dir = "None";

    let mut left_rear_speed: i32;
    let mut right_rear_speed: i32;
    let mut left_front_speed: i32;
    let mut right_front_speed: i32;

    let mut quit = false;
    let mut running = false;

    let mut gear = 1;
    control.set_gear(gear);
    control.set_bias(0);

    let mut decel = 0.0;

    context.pixel.all_on();
    context.pixel.render();

    context.display.clear();
    context
        .display
        .draw_text(4, 4, "Press start...", WHITE)
        .unwrap();
    context.display.update_all().unwrap();

    let mut current_colour = NONE;
    let mut previous_colour = NONE;

    while !quit {
        while let Some(event) = context.gilrs.next_event() {
            match event {
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::Start, _),
                    ..
                } => {
                    // Start button -> running
                    context.pixel.all_off();
                    context.pixel.render();

                    context.display.clear();
                    context
                        .display
                        .draw_text(4, 4, "Running       ", WHITE)
                        .unwrap();
                    context.display.update().unwrap();
                    running = true;
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::Mode, _),
                    ..
                } => {
                    println!("Mode....");
                    // Mode to exit
                    quit = true;
                    break;
                }
                _ => (),
            };
        }

        if running {
            let diff: i32 = 0;

            let front_dist = get_distance(&mut front, true);
            let right_dist = get_distance(&mut rightfront, true);
            let left_dist = get_distance(&mut leftfront, true);
            let back_dist = get_distance(&mut back, true);

            if direction == "Forward" {
                decel = get_deceleration(front_dist, MINDIST);
                if front_dist < MINDIST && right_dist < MAXDIST {
                    direction = "Left";
                }
            }

            if direction == "Left" {
                decel = get_deceleration(left_dist, MINDIST);
                if left_dist < MINDIST && back_dist < MAXDIST {                    
                    direction = "Forward";
                }
                if left_dist < MINDIST && front_dist < MAXDIST {
                    direction = "Back";
                }
            }

            if direction == "Back" {
                decel = get_deceleration(back_dist, MINDIST);
                if back_dist < MINDIST && left_dist < MAXDIST {
                    control.stop();
                    direction = "Right";
                }
                if back_dist < MINDIST && right_dist < MAXDIST {
                    control.stop();
                    direction = "Left";
                }
            }

            if direction == "Right" {
                decel = get_deceleration(right_dist, MINDIST);
                if right_dist < MINDIST {
                    direction = "Back";
                }
            }

            println!(
                "Direction {:#?} Fr {:#?}mm Lf {:#?}mm Rt {:#?}mm Bk {:#?}mm   Decel {:?}     ",
                direction, front_dist, left_dist, right_dist, back_dist, decel
            );

            if direction == "Forward" {
                let bias = 40;
                left_rear_speed = SPEED + bias;
                right_rear_speed = SPEED * -1;
                left_front_speed = SPEED + bias;
                right_front_speed = SPEED * -1;
                current_colour = GREEN;
            } else if direction == "Back" {
                let bias = 40;
                left_rear_speed = (SPEED + bias) * -1;
                right_rear_speed = SPEED;
                left_front_speed = (SPEED + bias) * -1;
                right_front_speed = SPEED;
                current_colour = RED;
            } else if direction == "Right" {
                // Strafe Right
                let bias = 30;
                left_front_speed = SPEED * -1;
                left_rear_speed = SPEED - bias;
                right_front_speed = SPEED * -1;
                right_rear_speed = SPEED - bias;
                current_colour = PURPLE;
            } else if direction == "Left" {
                // Strafe Left
                let bias = 10;
                left_front_speed = SPEED;
                left_rear_speed = (SPEED + bias) * -1;
                right_front_speed = SPEED;
                right_rear_speed = (SPEED + bias) * -1;
                current_colour = CYAN;
            } else {
                left_rear_speed = 0;
                right_rear_speed = 0;
                left_front_speed = 0;
                right_front_speed = 0;
                current_colour = NONE;
            }

            left_rear_speed = ((left_rear_speed as f64) * decel) as i32;
            right_rear_speed = ((right_rear_speed as f64) * decel) as i32;
            left_front_speed = ((left_front_speed as f64) * decel) as i32;
            right_front_speed = ((right_front_speed as f64) * decel) as i32;

            if current_colour != previous_colour {
                if current_colour == RED {
                    context.pixel.red();
                } else if current_colour == GREEN {
                    context.pixel.green();
                } else if current_colour == BLUE {
                    context.pixel.blue();
                } else if current_colour == YELLOW {
                    context.pixel.yellow();
                } else if current_colour == PURPLE {
                    context.pixel.purple();
                } else if current_colour == CYAN {
                    context.pixel.cyan();
                } else if current_colour == ALL {
                    context.pixel.white();
                } else if current_colour == NONE {
                    context.pixel.all_off();
                }
                context.pixel.render();
                previous_colour = current_colour;
            }

            println!(
                "Speeds lf {:?}, lr {:#?}, rf {:#?}, rr {:#?}",
                left_front_speed, left_rear_speed, right_front_speed, right_rear_speed
            );
            control.speed(
                left_rear_speed,
                right_rear_speed,
                left_front_speed,
                right_front_speed,
            );
        }
    }

    context.pixel.all_off();
    context.pixel.render();

    control.stop();
    context.display.clear();
}

fn print_colour(context: &mut Context, colour: i32) -> &str {
    match colour {
        RED => {
            println!("Found Red!");
            context.pixel.red();
            context.pixel.render();
            return "Red";
        }
        BLUE => {
            println!("Found Blue!");
            context.pixel.blue();
            context.pixel.render();
            return "Blue";
        }
        YELLOW => {
            println!("Found Yellow!");
            context.pixel.yellow();
            context.pixel.render();
            return "Yellow";
        }
        GREEN => {
            println!("Found Green!");
            context.pixel.green();
            context.pixel.render();
            return "Green";
        }
        _ => {
            println!("Found Unknown");
            context.pixel.all_off();
            context.pixel.render();
            return "unknown";
        }
    }
}

fn do_hubble(context: &mut Context, mut locations: [f32; 4], mut order: [i32; 4]){
    
    const MINIMUM : u16 = 200;
    let interval = time::Duration::from_millis(50);
    
    // Distance sensors
    let mut front = try_open_tof("/dev/i2c-8");
    let mut back = try_open_tof("/dev/i2c-7");
    
    println!("front started");
    println!("back started");
    println!("right back started");
    println!("left back started");

    set_continous(&mut front);
    set_continous(&mut back);
    
    let mut front_dist = get_distance(&mut front, true);
    let mut back_dist = get_distance(&mut back, true);
    
    let mut leftback = try_open_tof("/dev/i2c-6");
    let mut rightback = try_open_tof("/dev/i2c-9");
    
    set_continous(&mut leftback);
    set_continous(&mut rightback);
    
    let mut right_dist = get_distance(&mut rightback, true);
    let mut left_dist = get_distance(&mut leftback, true);

    let mut control = build_control();
    control.init();

    let mut distance: u16 = 0;
    let mut direction = "Start";
    let mut prev_dir = "None";

    let mut left_rear_speed: i32;
    let mut right_rear_speed: i32;
    let mut left_front_speed: i32;
    let mut right_front_speed: i32;

    let mut quit = false;
    let mut running = false;

    let mut gear = 1;
    control.set_gear(gear);
    control.set_bias(0);

    let mut decel = 0.0;

    context.pixel.all_on();
    context.pixel.render();

    context.display.clear();
    context
        .display
        .draw_text(4, 4, "Press start...", WHITE)
        .unwrap();
    context.display.update_all().unwrap();

    let mut current_colour = NONE;
    let mut previous_colour = NONE;

    while !quit {
        while let Some(event) = context.gilrs.next_event() {
            match event {
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::Start, _),
                    ..
                } => {
                    // Start button -> running
                    context.pixel.all_off();
                    context.pixel.render();

                    context.display.clear();
                    context
                        .display
                        .draw_text(4, 4, "Running       ", WHITE)
                        .unwrap();
                    context.display.update().unwrap();
                    running = true;
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::Mode, _),
                    ..
                } => {
                    println!("Mode....");
                    // Mode to exit
                    quit = true;
                    break;
                }
                _ => (),
            };
        }

        if running {
            let diff: i32 = 0;

            front_dist = get_distance(&mut front, true);
            right_dist = get_distance(&mut rightback, true);
            left_dist = get_distance(&mut leftback, true);
            back_dist = get_distance(&mut back, true);

            // Hubble
            if direction == "Start" {
                decel = get_deceleration(front_dist, MINIMUM);
                if front_dist < MINIMUM {
                    direction = "Left";
                }
            }

            if direction == "Left" {
                decel = get_deceleration(left_dist, MINIMUM);                
                if left_dist < MINIMUM {
                    direction = "Back";
                }
            }

            if direction == "Back" {
                decel = get_deceleration(back_dist, MINIMUM);
                if back_dist < MINIMUM {                    
                    direction = "Right";
                }
            }

            if direction == "Right" {
                decel = get_deceleration(right_dist, MINIMUM);
                if right_dist < MINIMUM {
                    direction = "Forward";
                }
            }
            
            if direction == "Forward" {
                decel = get_deceleration(front_dist, MINIMUM);
                if front_dist < MINIMUM {
                    direction = "Stop";
                }
            }

            println!(
                "Direction {:#?} Fr {:#?}mm Lf {:#?}mm Rt {:#?}mm Bk {:#?}mm   Decel {:?}     ",
                direction, front_dist, left_dist, right_dist, back_dist, decel
            );

            if direction == "Start" || direction == "Forward" {
                let bias = 40;
                left_rear_speed = SPEED + bias;
                right_rear_speed = SPEED * -1;
                left_front_speed = SPEED + bias;
                right_front_speed = SPEED * -1;
                current_colour = GREEN;
            } else if direction == "Back" {
                let bias = 40;
                left_rear_speed = (SPEED + bias) * -1;
                right_rear_speed = SPEED;
                left_front_speed = (SPEED + bias) * -1;
                right_front_speed = SPEED;
                current_colour = RED;
            } else if direction == "Right" {
                // Strafe Right
                let bias = 30;
                left_front_speed = SPEED * -1;
                left_rear_speed = SPEED - bias;
                right_front_speed = SPEED * -1;
                right_rear_speed = SPEED - bias;
                current_colour = PURPLE;
            } else if direction == "Left" {
                // Strafe Left
                let bias = 10;
                left_front_speed = SPEED;
                left_rear_speed = (SPEED + bias) * -1;
                right_front_speed = SPEED;
                right_rear_speed = (SPEED + bias) * -1;
                current_colour = CYAN;
            } else {
                left_rear_speed = 0;
                right_rear_speed = 0;
                left_front_speed = 0;
                right_front_speed = 0;
                current_colour = NONE;
            }

            left_rear_speed = ((left_rear_speed as f64) * decel) as i32;
            right_rear_speed = ((right_rear_speed as f64) * decel) as i32;
            left_front_speed = ((left_front_speed as f64) * decel) as i32;
            right_front_speed = ((right_front_speed as f64) * decel) as i32;

            if current_colour != previous_colour {
                if current_colour == RED {
                    context.pixel.red();
                } else if current_colour == GREEN {
                    context.pixel.green();
                } else if current_colour == BLUE {
                    context.pixel.blue();
                } else if current_colour == YELLOW {
                    context.pixel.yellow();
                } else if current_colour == PURPLE {
                    context.pixel.purple();
                } else if current_colour == CYAN {
                    context.pixel.cyan();
                } else if current_colour == ALL {
                    context.pixel.white();
                } else if current_colour == NONE {
                    context.pixel.all_off();
                }
                context.pixel.render();
                previous_colour = current_colour;
            }

            //println!(
                //"Speeds lf {:?}, lr {:#?}, rf {:#?}, rr {:#?}",
                //left_front_speed, left_rear_speed, right_front_speed, right_rear_speed
            //);
            control.speed(
                left_rear_speed,
                right_rear_speed,
                left_front_speed,
                right_front_speed,
            );
        }
    }

    context.pixel.all_off();
    context.pixel.render();

    control.stop();
    context.display.clear();
}

use std::sync::{Arc, Mutex};

fn _do_hubble(context: &mut Context, mut locations: [f32; 4], mut order: [i32; 4]) {
    const DRIVING_SPEED: i32 = 1000;
    const TURNING_SPEED: i32 = 400;
    const MIN_DIST: u16 = 100;
    const MAX_DIST: u16 = 600;
    
    let mut left_rear_speed: i32;
    let mut right_rear_speed: i32;
    let mut left_front_speed: i32;
    let mut right_front_speed: i32;

    let interval = time::Duration::from_millis(2000);

    context.pixel.all_on();
    let mut control = build_control();
    control.init();

    let mut pos = 0;

    //let mut compass = HMC5883L::new("/dev/i2c-1").unwrap();

    let mut front = try_open_tof("/dev/i2c-8");
    let mut back = try_open_tof("/dev/i2c-7");
    println!("front started");
    println!("back started");

    set_continous(&mut front);
    set_continous(&mut back);

    context.display.clear();
    context
        .display
        .draw_text(4, 4, "Press Left(E)...", WHITE)
        .unwrap();
    context.display.update_all().unwrap();

    let mut gear = 4;
    control.set_gear(gear);
    control.set_bias(0);

    let mut running = false;
    let mut quit = false;

    //let mut heading = compass.read_degrees().unwrap();
    //let mut target = compass.read_degrees().unwrap();
    
    let colour: i32 = NONE;
    let shared = Arc::new(colour);
    let (command_tx, command_rx) = mpsc::channel();
    
    let t = thread::spawn(move || {
        let mut col = Arc::clone(&shared);
        println!("Thread Starting");        
        let mut cam = build_camera();
        load_calibration(&mut cam);
        cam.discard_video();        
        
        loop {     
            let value = cam.get_colour(false);      
            match command_rx.try_recv() {
                Ok("X") | Err(TryRecvError::Disconnected) => {
                    println!("Terminating.");
                    break;
                }                
                Ok(&_) | Err(TryRecvError::Empty) => {}
            }            
        }
    });

    while !quit {
        

        while let Some(event) = context.gilrs.next_event() {
            match event {
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::East, _),
                    ..
                } => {
                    println!("East Pressed");
                    // Start button -> running
                    context.pixel.all_off();
                    context
                        .display
                        .draw_text(4, 4, "              ", WHITE)
                        .unwrap();
                    context.display.update().unwrap();
                    running = true;
                }
                // Needs gear changing here
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::Mode, _),
                    ..
                } => {
                    println!("Mode");
                    // Mode to exit
                    let _ = command_tx.send("X");
                    quit = true;
                    break;
                }
                _ => (),
            };
        }

        // Main State running or not
        //if running {
            //let mut front_dist = get_distance(&mut front, true);
            //let mut back_dist = get_distance(&mut back, true);

            //let colour = cam.get_colour(true);
            //heading = compass.read_degrees().unwrap();
            //// first time through && locations[index] == 0.0
            //if order[0] == NONE || order[1] == NONE || order[2] == NONE || order[3] == NONE {
                //if colour == RED || colour == BLUE || colour == YELLOW || colour == GREEN {
                    //print_colour(context, colour);
                    //let index = colour as usize;
                    //println!("Index {}", index);
                    //if locations[index] == 0.0 {
                        //control.stop();
                        //println!("Heading {}", heading);
                        //locations[index] = heading;
                        //order[pos] = colour;
                        //print_colour(context, colour);
                        //pos = pos + 1;
                    //}
                //}
                //control.turn_left(TURNING_SPEED, gear);
            //} else {
                //println!("Locations: {:#?}", locations);
                //running = false;
                //for i in RED..GREEN {
                    //let index = i as usize;
                    //println!("Searching for {:#?}", print_colour(context, colour));
                    //align(locations[index], &mut compass, &mut control, &mut cam, 4);
                    //loop {
                        //cam.discard_video();
                        //front_dist = get_distance(&mut front, true);
                        //back_dist = get_distance(&mut back, true);
                        //if front_dist < MIN_DIST {
                            //control.stop();
                            //break;
                        //}
                        //let decel = get_deceleration(front_dist);
                        //let bias = 50;
                        //left_rear_speed = DRIVING_SPEED + bias;
                        //right_rear_speed = DRIVING_SPEED * -1;
                        //left_front_speed = DRIVING_SPEED + bias;
                        //right_front_speed = DRIVING_SPEED * -1;
                        //left_rear_speed = ((left_rear_speed as f64) * decel) as i32;
                        //right_rear_speed = ((right_rear_speed as f64) * decel) as i32;
                        //left_front_speed = ((left_front_speed as f64) * decel) as i32;
                        //right_front_speed = ((right_front_speed as f64) * decel) as i32;
                        ////println!(
                            ////"Forward Speeds lf {:?}, lr {:#?}, rf {:#?}, rr {:#?}",
                            ////left_front_speed, left_rear_speed, right_front_speed, right_rear_speed
                        ////);

                        //control.speed(
                            //left_rear_speed,
                            //right_rear_speed,
                            //left_front_speed,
                            //right_front_speed,
                        //);
                    //}
                    //println!("Backward");
                    //loop {
                        //cam.discard_video();
                        //front_dist = get_distance(&mut front, true);
                        //back_dist = get_distance(&mut back, true);
                        //if front_dist > MAX_DIST {
                            //control.stop();
                            //break;
                        //}
                        //let decel = get_deceleration(back_dist);
                        //let bias = 50;
                        //left_rear_speed = (DRIVING_SPEED + bias) * -1;
                        //right_rear_speed = DRIVING_SPEED;
                        //left_front_speed = (DRIVING_SPEED + bias) * -1;
                        //right_front_speed = DRIVING_SPEED;
                        //left_rear_speed = ((left_rear_speed as f64) * decel) as i32;
                        //right_rear_speed = ((right_rear_speed as f64) * decel) as i32;
                        //left_front_speed = ((left_front_speed as f64) * decel) as i32;
                        //right_front_speed = ((right_front_speed as f64) * decel) as i32;
                        ////println!(
                            ////"Backward Speeds lf {:?}, lr {:#?}, rf {:#?}, rr {:#?}",
                            ////left_front_speed, left_rear_speed, right_front_speed, right_rear_speed
                        ////);

                        //control.speed(
                            //left_rear_speed,
                            //right_rear_speed,
                            //left_front_speed,
                            //right_front_speed,
                        //);
                    //}
                //}
            //}
        //}
    
    }

    
    control.stop();
    context.display.clear();
    context.pixel.all_off();
    thread::sleep(interval);
}

fn do_straight(context: &mut Context) {
    let interval = time::Duration::from_millis(2000);

    let mut control = build_control();
    control.init();
    control.set_gear(2);

    let mut left = VL53L0X::new("/dev/i2c-5").unwrap();
    let mut right = VL53L0X::new("/dev/i2c-10").unwrap();

    context.pixel.all_on();
    context.pixel.render();

    context.display.clear();
    context
        .display
        .draw_text(4, 4, "Press start...", WHITE)
        .unwrap();
    context.display.update_all().unwrap();

    let mut target: i32 = 0;

    let mut quit = false;
    let mut running = false;
    while !quit {
        while let Some(event) = context.gilrs.next_event() {
            match event {
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::Start, _),
                    ..
                } => {
                    println!("Select Pressed");
                    // Start button -> running
                    context.pixel.all_off();
                    target = left.read() as i32 - right.read() as i32;
                    context
                        .display
                        .draw_text(4, 4, "              ", WHITE)
                        .unwrap();
                    context.display.update().unwrap();
                    println!("Target {:?}", target);
                    running = true;
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::Mode, _),
                    ..
                } => {
                    println!("Mode....");
                    // Mode to exit
                    quit = true;
                    break;
                }
                _ => (),
            };
        }

        if running {
            let mut left_rear_speed: i32 = 1000;
            let mut right_rear_speed: i32 = -1000;
            let mut left_front_speed: i32 = 1000;
            let mut right_front_speed: i32 = -1000;

            let right_dist: i32 = right.read() as i32;
            let left_dist: i32 = left.read() as i32;

            println!(
                "Target {:#?}mm, Right {:#?}mm, Left {:#?}mm ",
                target, right_dist, left_dist
            );

            let difference: i32 = (target - (left_dist - right_dist)) * 5;

            if difference > 15 {
                // turn right
                context.pixel.right_red();
                context.pixel.render();
                println!("Turn Right {:04}  ", difference);
                left_front_speed = left_front_speed; //+ difference;
                left_rear_speed = left_rear_speed; //+ difference;
                right_front_speed = right_front_speed + difference;
                right_rear_speed = right_rear_speed + difference;
            } else if difference < -15 {
                // turn left
                context.pixel.left_red();
                context.pixel.render();
                println!("Turn Left  {:04}  ", -difference);
                left_front_speed = left_front_speed + difference;
                left_rear_speed = left_rear_speed + difference;
                right_front_speed = right_front_speed; //+ difference;
                right_rear_speed = right_rear_speed; //+ difference;
            } else {
                //println!("Straight");
                context.pixel.all_off();
                context.pixel.render();
            }

            {
                //if left_rear_speed != 0 || right_rear_speed != 0 || left_front_speed != 0 || right_front_speed != 0  {
                //println!(" {0}, {1}, {2}, {3} ", left_rear_speed, right_rear_speed, left_front_speed, right_front_speed );
                //}
            }
            control.speed(
                left_rear_speed,
                right_rear_speed,
                left_front_speed,
                right_front_speed,
            );
        }
    }

    control.stop();

    context.display.clear();
    context.pixel.all_off();
    thread::sleep(interval);
}

fn do_wheels_rc(context: &mut Context) {
    const DEADZONE: i32 = 50;

    let mut control = build_control();
    control.init();

    let servo = build_servo(21);

    let mut gear = 1;
    let mut quit = false;
    let mut left_stick_y = 0;
    let mut right_stick_y = 0;

    let mut current_colour = NONE;
    let mut previous_colour = NONE;

    while !quit {
        while let Some(event) = context.gilrs.next_event() {
            match event {
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::Mode, _),
                    ..
                } => {
                    println!("Mode Pressed");
                    quit = true;
                    break;
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::DPadUp, _),
                    ..
                } => {
                    println!("DPad Up Pressed");
                    servo.set_pulse_width(2500);
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::DPadDown, _),
                    ..
                } => {
                    println!("DPad Up Pressed");
                    servo.set_pulse_width(500);
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::North, _),
                    ..
                } => {
                    gear = 1;
                    context
                        .display
                        .draw_text(4, 4, &gear.to_string(), LT_GREY)
                        .unwrap();
                    context.display.update().unwrap();
                    println!(" {0} ", gear);
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::West, _),
                    ..
                } => {
                    gear = 2;
                    context
                        .display
                        .draw_text(4, 4, &gear.to_string(), LT_GREY)
                        .unwrap();
                    context.display.update().unwrap();
                    println!(" {0} ", gear);
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::East, _),
                    ..
                } => {
                    gear = 3;
                    context
                        .display
                        .draw_text(4, 4, &gear.to_string(), LT_GREY)
                        .unwrap();
                    context.display.update().unwrap();
                    println!(" {0} ", gear);
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::South, _),
                    ..
                } => {
                    gear = 4;
                    context
                        .display
                        .draw_text(4, 4, &gear.to_string(), LT_GREY)
                        .unwrap();
                    context.display.update().unwrap();
                    println!(" {0} ", gear);
                }
                Event {
                    id: _,
                    event: EventType::AxisChanged(LeftStickY, value, _),
                    ..
                } => {
                    //println!("Left Stick Y {:?}", value);
                    left_stick_y = (value * 1000.0) as i32;
                }
                Event {
                    id: _,
                    event: EventType::AxisChanged(RightStickY, value, _),
                    ..
                } => {
                    //println!("Right Stick Y {:?}", value);
                    right_stick_y = (value * 1000.0) as i32;
                }
                _ => {
                    break;
                }
            };

            let mut left_rear_speed: i32;
            let mut right_rear_speed: i32;
            let mut left_front_speed: i32;
            let mut right_front_speed: i32;

            if left_stick_y > DEADZONE && right_stick_y > DEADZONE {
                // Forward
                left_front_speed = left_stick_y;
                left_rear_speed = left_stick_y;
                right_front_speed = -right_stick_y;
                right_rear_speed = -right_stick_y;
                current_colour = GREEN;
            } else if left_stick_y < -DEADZONE && right_stick_y < -DEADZONE {
                // Backwards
                left_front_speed = left_stick_y;
                left_rear_speed = left_stick_y;
                right_front_speed = -right_stick_y;
                right_rear_speed = -right_stick_y;
                current_colour = RED;
            } else if left_stick_y > DEADZONE && right_stick_y < -DEADZONE {
                // Turn Sharp Right
                left_front_speed = left_stick_y;
                left_rear_speed = left_stick_y;
                right_front_speed = -right_stick_y;
                right_rear_speed = -right_stick_y;
                current_colour = YELLOW;
            } else if left_stick_y < -DEADZONE && right_stick_y > DEADZONE {
                // Turn Sharp Left
                left_front_speed = left_stick_y;
                left_rear_speed = left_stick_y;
                right_front_speed = -right_stick_y;
                right_rear_speed = -right_stick_y;
                current_colour = BLUE;
            } else if left_stick_y > DEADZONE && right_stick_y == 0 {
                // Turn Right
                left_front_speed = left_stick_y;
                left_rear_speed = left_stick_y;
                right_front_speed = -right_stick_y;
                right_rear_speed = -right_stick_y;
                current_colour = YELLOW;
            } else if left_stick_y == 0 && right_stick_y > DEADZONE {
                // Turn Left
                left_front_speed = left_stick_y;
                left_rear_speed = left_stick_y;
                right_front_speed = -right_stick_y;
                right_rear_speed = -right_stick_y;
                current_colour = BLUE;
            } else {
                left_rear_speed = 0;
                right_rear_speed = 0;
                left_front_speed = 0;
                right_front_speed = 0;
                current_colour = NONE;
            }

            left_front_speed = left_front_speed / gear;
            right_front_speed = right_front_speed / gear;
            left_rear_speed = left_rear_speed / gear;
            right_rear_speed = right_rear_speed / gear;

            if left_rear_speed != 0
                || right_rear_speed != 0
                || left_front_speed != 0
                || right_front_speed != 0
            {
                println!(
                    "Speed left rear: {0}, right rear: {1}, left front: {2} right front: {3}",
                    left_rear_speed, right_rear_speed, left_front_speed, right_front_speed
                );
            }

            if current_colour != previous_colour {
                if current_colour == RED {
                    context.pixel.red();
                } else if current_colour == GREEN {
                    context.pixel.green();
                } else if current_colour == BLUE {
                    context.pixel.blue();
                } else if current_colour == YELLOW {
                    context.pixel.yellow();
                } else if current_colour == PURPLE {
                    context.pixel.purple();
                } else if current_colour == CYAN {
                    context.pixel.cyan();
                } else if current_colour == ALL {
                    context.pixel.white();
                } else if current_colour == NONE {
                    context.pixel.all_off();
                }
                context.pixel.render();
                previous_colour = current_colour;
            }
            control.speed(
                left_rear_speed,
                right_rear_speed,
                left_front_speed,
                right_front_speed,
            );
        }
    }

    control.stop();
    context.display.clear();
}

fn do_mecanum_rc(context: &mut Context) {
    const DEADZONE: i32 = 200;

    let mut control = build_control();
    control.init();

    let servo = build_servo(21);

    let mut gear = 3;
    control.set_gear(gear);

    let mut left_stick_x = 0;
    let mut left_stick_y = 0;
    let mut right_stick_y = 0;
    let mut right_stick_x = 0;

    let mut current_colour = NONE;
    let mut previous_colour = NONE;

    let mut dpad = 0;
    let mut quit = false;
    while !quit {
        while let Some(event) = context.gilrs.next_event() {
            match event {
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::Mode, _),
                    ..
                } => {
                    println!("Mode Pressed");
                    quit = true;
                    break;
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::DPadUp, _),
                    ..
                } => {
                    println!("DPad Up Pressed");
                    servo.set_pulse_width(2500);
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::DPadDown, _),
                    ..
                } => {
                    println!("DPad Up Pressed");
                    servo.set_pulse_width(500);
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::North, _),
                    ..
                } => {
                    gear = 1;
                    context
                        .display
                        .draw_text(4, 4, &gear.to_string(), LT_GREY)
                        .unwrap();
                    context.display.update().unwrap();
                    println!(" {0} ", gear);
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::West, _),
                    ..
                } => {
                    gear = 2;
                    context
                        .display
                        .draw_text(4, 4, &gear.to_string(), LT_GREY)
                        .unwrap();
                    context.display.update().unwrap();
                    println!(" {0} ", gear);
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::East, _),
                    ..
                } => {
                    gear = 3;
                    context
                        .display
                        .draw_text(4, 4, &gear.to_string(), LT_GREY)
                        .unwrap();
                    context.display.update().unwrap();
                    println!(" {0} ", gear);
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::South, _),
                    ..
                } => {
                    gear = 4;
                    context
                        .display
                        .draw_text(4, 4, &gear.to_string(), LT_GREY)
                        .unwrap();
                    context.display.update().unwrap();
                    println!(" {0} ", gear);
                }
                Event {
                    id: _,
                    event: EventType::AxisChanged(LeftStickY, value, _),
                    ..
                } => {
                    //println!("Left Stick Y {:?}", value);
                    left_stick_y = (value * 1000.0) as i32;
                }
                Event {
                    id: _,
                    event: EventType::AxisChanged(LeftStickX, value, _),
                    ..
                } => {
                    //println!("Left Stick X {:?}", value);
                    left_stick_x = (value * 1000.0) as i32;
                }
                Event {
                    id: _,
                    event: EventType::AxisChanged(RightStickY, value, _),
                    ..
                } => {
                    //println!("Right Stick Y {:?}", value);
                    right_stick_y = (value * 1000.0) as i32;
                }
                Event {
                    id: _,
                    event: EventType::AxisChanged(RightStickX, value, _),
                    ..
                } => {
                    //println!("Right Stick X {:?}", value);
                    right_stick_x = (value * 1000.0) as i32;
                }
                _ => {
                    break;
                }
            };

            let mut left_rear_speed: i32;
            let mut right_rear_speed: i32;
            let mut left_front_speed: i32;
            let mut right_front_speed: i32;

            if left_stick_y > DEADZONE && right_stick_y > DEADZONE {
                // Forward
                left_front_speed = left_stick_y;
                left_rear_speed = left_stick_y;
                right_front_speed = -right_stick_y;
                right_rear_speed = -right_stick_y;
                current_colour = GREEN;
            } else if left_stick_y < -DEADZONE && right_stick_y < -DEADZONE {
                // Backwards
                left_front_speed = left_stick_y;
                left_rear_speed = left_stick_y;
                right_front_speed = -right_stick_y;
                right_rear_speed = -right_stick_y;
                current_colour = RED;
            } else if left_stick_y > DEADZONE && right_stick_y < -DEADZONE {
                // Turn Right
                left_front_speed = left_stick_y;
                left_rear_speed = left_stick_y;
                right_front_speed = -right_stick_y;
                right_rear_speed = -right_stick_y;
                current_colour = YELLOW;
            } else if left_stick_y < -DEADZONE && right_stick_y > DEADZONE {
                // Turn Left
                left_front_speed = left_stick_y;
                left_rear_speed = left_stick_y;
                right_front_speed = -right_stick_y;
                right_rear_speed = -right_stick_y;
                current_colour = BLUE;
            } else if left_stick_x < -DEADZONE && right_stick_x < -DEADZONE {
                // Strafe left
                left_front_speed = -left_stick_x;
                left_rear_speed = left_stick_x;
                right_front_speed = -right_stick_x;
                right_rear_speed = right_stick_x;
                current_colour = PURPLE;
            } else if left_stick_x > DEADZONE && right_stick_x > DEADZONE {
                // Strafe Right
                left_front_speed = -left_stick_x;
                left_rear_speed = left_stick_x;
                right_front_speed = -right_stick_x;
                right_rear_speed = right_stick_x;
                current_colour = CYAN;
            } else {
                left_rear_speed = 0;
                right_rear_speed = 0;
                left_front_speed = 0;
                right_front_speed = 0;
                current_colour = NONE;
            }

            if left_rear_speed != 0
                || right_rear_speed != 0
                || left_front_speed != 0
                || right_front_speed != 0
            {
                println!(
                    "Stick left XY: {0},{1}   right X:Y {2},{3}",
                    left_stick_x, left_stick_y, right_stick_x, right_stick_y
                );
                println!(
                    "Speed left rear: {0}, right rear: {1}, left front: {2} right front: {3}",
                    left_rear_speed, right_rear_speed, left_front_speed, right_front_speed
                );
            }

            if current_colour != previous_colour {
                if current_colour == RED {
                    context.pixel.red();
                } else if current_colour == GREEN {
                    context.pixel.green();
                } else if current_colour == BLUE {
                    context.pixel.blue();
                } else if current_colour == YELLOW {
                    context.pixel.yellow();
                } else if current_colour == PURPLE {
                    context.pixel.purple();
                } else if current_colour == CYAN {
                    context.pixel.cyan();
                } else if current_colour == ALL {
                    context.pixel.white();
                } else if current_colour == NONE {
                    context.pixel.all_off();
                }
                context.pixel.render();
                previous_colour = current_colour;
            }

            control.set_gear(gear);
            control.speed(
                left_rear_speed,
                right_rear_speed,
                left_front_speed,
                right_front_speed,
            );
        }
    }

    control.stop();
    context.pixel.all_off();
    context.display.clear();
}

fn try_open_tof(filename: &'static str) -> Option<VL53L0X> {
    let front = match VL53L0X::new(filename) {
        Ok(front) => front,
        Err(e) => {
            println!("Failed to open front TOF {:?} ", e);
            return None;
        }
    };
    println!("Success {:?}", filename);
    return Some(front);
}

fn get_distance(tof: &mut Option<VL53L0X>, continous: bool) -> u16 {
    let dist: u16;
    if continous {
        match tof {
            None => dist = 0,
            Some(ref mut tof) => {
                dist = tof.read_continous();
            }
        }
    } else {
        match tof {
            None => dist = 0,
            Some(ref mut tof) => {
                dist = tof.read();
            }
        }
    }
    return dist;
}

fn set_continous(tof: &mut Option<VL53L0X>) {
    match tof {
        None => (),
        Some(ref mut tof) => match tof.start_continuous() {
            Ok(()) => {
                println!("Set continuous");
            }
            Err(e) => {
                println!("Failed to set continuous {:?}", e);
            }
        },
    }
}

fn do_run_tests(context: &mut Context) {
    //let mut cam = build_camera();
    //load_calibration( &mut cam );

    // Test compass
    //let mut compass = HMC5883L::new("/dev/i2c-1").unwrap();
    //println!("Compass started");

    // Test distance sensors group 1 (not always present)
    let mut front = try_open_tof("/dev/i2c-8");
    let mut leftfront = try_open_tof("/dev/i2c-5");
    let mut rightfront = try_open_tof("/dev/i2c-10");

    // Test distance sensors group 2
    let mut back = try_open_tof("/dev/i2c-7");
    let mut leftback = try_open_tof("/dev/i2c-6");
    let mut rightback = try_open_tof("/dev/i2c-9");

    //let mut heading = compass.read_degrees().unwrap();

    set_continous(&mut back);
    set_continous(&mut leftback);
    set_continous(&mut rightback);
    let mut bk_dist = get_distance(&mut back, true);
    let mut lb_dist = get_distance(&mut leftback, true);
    let mut rb_dist = get_distance(&mut rightback, true);

    set_continous(&mut front);
    let mut ft_dist = get_distance(&mut front, true);

    set_continous(&mut leftfront);
    set_continous(&mut rightfront);
    let mut lf_dist = get_distance(&mut leftfront, true);
    let mut rf_dist = get_distance(&mut rightfront, true);

    let mut colour_visible = 0;
    context.pixel.all_on();
    context.pixel.render();

    let interval = Duration::from_millis(200);
    let mut now = Instant::now();

    let (command_tx, command_rx) = mpsc::channel();
    let (data_tx, data_rx) = mpsc::channel();

    let t = thread::spawn(move || {
        println!("Thread Starting");
        let colour: i32 = 0;
        let mut cam = build_camera();
        load_calibration(&mut cam);
        loop {
            cam.discard_video();
            match command_rx.try_recv() {
                Ok("X") | Err(TryRecvError::Disconnected) => {
                    println!("Terminating.");
                    break;
                }
                Ok("F") => {
                    let colour = cam.get_colour(false);
                    if colour == RED {
                        let _ = data_tx.send("0");
                    }
                    if colour == BLUE {
                        let _ = data_tx.send("1");
                    }
                    if colour == YELLOW {
                        let _ = data_tx.send("2");
                    }
                    if colour == GREEN {
                        let _ = data_tx.send("3");
                    }
                }
                Ok(&_) | Err(TryRecvError::Empty) => {}
            }
        }
    });

    let mut quit = false;
    while !quit {
        match data_rx.try_recv() {
            Ok("0") => {
                print_colour(context, RED);
                colour_visible = RED;
            }
            Ok("1") => {
                print_colour(context, BLUE);
                colour_visible = BLUE;
            }
            Ok("2") => {
                print_colour(context, YELLOW);
                colour_visible = YELLOW;
            }
            Ok("3") => {
                print_colour(context, GREEN);
                colour_visible = GREEN;
            }
            Ok(_) | Err(_) => {}
        }

        while let Some(event) = context.gilrs.next_event() {
            context.gilrs.update(&event);
            match event {
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::Mode, _),
                    ..
                } => {
                    //println!("Mode Pressed");
                    quit = true;
                    break;
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::North, _),
                    ..
                } => {
                    //println!("North Pressed");
                    //let colour_visible = cam.get_colour( false );
                    let _ = command_tx.send("F");
                    //heading = compass.read_degrees().unwrap();
                    bk_dist = get_distance(&mut back, true);
                    lb_dist = get_distance(&mut leftback, true);
                    rb_dist = get_distance(&mut rightback, true);

                    ft_dist = get_distance(&mut front, true);
                    lf_dist = get_distance(&mut leftfront, true);
                    rf_dist = get_distance(&mut rightfront, true);

                    context.display.clear();
                    //context
                        //.display
                        //.draw_text(0, 8, "Head:           ", WHITE)
                        //.unwrap();
                    //context
                        //.display
                        //.draw_text(56, 8, &format!("{:5.2} ", heading), WHITE)
                        //.unwrap();
                    context.display.draw_text(0, 16, "LB:", WHITE).unwrap();
                    context
                        .display
                        .draw_text(56, 16, &format!("{:5.2} ", lb_dist), WHITE)
                        .unwrap();
                    context.display.draw_text(0, 24, "RB:", WHITE).unwrap();
                    context
                        .display
                        .draw_text(56, 24, &format!("{:5.2} ", rb_dist), WHITE)
                        .unwrap();
                    context.display.draw_text(0, 40, "Back:", WHITE).unwrap();
                    context
                        .display
                        .draw_text(56, 40, &format!("{:5.2} ", bk_dist), WHITE)
                        .unwrap();

                    context.display.draw_text(0, 32, "Front:", WHITE).unwrap();
                    context
                        .display
                        .draw_text(56, 32, &format!("{:5.2} ", ft_dist), WHITE)
                        .unwrap();
                    context.display.draw_text(0, 48, "LF:", WHITE).unwrap();
                    context
                        .display
                        .draw_text(56, 48, &format!("{:5.2} ", lf_dist), WHITE)
                        .unwrap();
                    context.display.draw_text(0, 56, "RF:", WHITE).unwrap();
                    context
                        .display
                        .draw_text(56, 56, &format!("{:5.2} ", rf_dist), WHITE)
                        .unwrap();
                    context.display.draw_text(0, 64, "Colour:", WHITE).unwrap();
                    context.display.update().unwrap();
                    break;
                }
                _ => {
                    break;
                }
            };
        }
        //heading = compass.read_degrees().unwrap();
        ft_dist = get_distance(&mut front, true);
        bk_dist = get_distance(&mut back, true);
        lb_dist = get_distance(&mut leftback, true);
        rb_dist = get_distance(&mut rightback, true);
        lf_dist = get_distance(&mut leftfront, true);
        rf_dist = get_distance(&mut rightfront, true);

        //println!("Current Heading      {:5.2}  ", heading);
        println!("Left Back Distance   {:5.2}  ", lb_dist);
        println!("Back Distance        {:5.2}  ", bk_dist);
        println!("Front Distance       {:5.2}  ", ft_dist);
        println!("Right Back Distance  {:5.2}  ", rb_dist);
        println!("Left Front Distance  {:5.2}  ", lf_dist);
        println!("Right Front Distance {:5.2}  ", rf_dist);
    }
    context.pixel.all_off();
    context.pixel.render();
}

fn load_calibration(cam: &mut Camera) {
    let file = fs::File::open("calibrate.json").expect("file should open read only");
    let mut calibrate: Calibrate =
        serde_json::from_reader(file).expect("file should be proper JSON");

    println!("Calibrate {:?}", calibrate);
    cam.set_red_lower(&mut calibrate.red_lower);
    cam.set_red_upper(&mut calibrate.red_upper);
    cam.set_green_lower(&mut calibrate.green_lower);
    cam.set_green_upper(&mut calibrate.green_upper);
    cam.set_blue_lower(&mut calibrate.blue_lower);
    cam.set_blue_upper(&mut calibrate.blue_upper);
    cam.set_yellow_lower(&mut calibrate.yellow_lower);
    cam.set_yellow_upper(&mut calibrate.yellow_upper);

    cam.dump_bounds();
}

fn do_calibrate(context: &mut Context) {
    
    const DIST: u16 = 300;
    context.pixel.all_off();
    context.pixel.render();

    //let mut compass = HMC5883L::new("/dev/i2c-1").unwrap();
    let mut front = try_open_tof("/dev/i2c-8");
    let mut leftfront = try_open_tof("/dev/i2c-5");
    let mut rightfront = try_open_tof("/dev/i2c-10");
    let mut back = try_open_tof("/dev/i2c-7");
    println!("front started");
    println!("left front started");
    println!("right front started");
    println!("back started");

    set_continous(&mut front);
    set_continous(&mut leftfront);
    set_continous(&mut rightfront);
    set_continous(&mut back);

    let mut control = build_control();
    control.init();

    let mut distance: u16 = 0;
    let mut direction = "Front";
    let mut diff: i32 = 0;

    //let original = compass.read_degrees().unwrap();
    //let mut heading = compass.read_degrees().unwrap();

    let mut left_rear_speed: i32 = 0;
    let mut right_rear_speed: i32 = 0;
    let mut left_front_speed: i32 = 0;
    let mut right_front_speed: i32 = 0;

    let mut quit = false;
    let mut running = false;
    let mut direction = "North";

    let mut gear = 1;
    control.set_gear(gear);
    control.set_bias(0);

    let mut decel = 0.0;

    //println!(
        //"Init Original {:#?}°,  Heading {:#?}° Diff {:#?} Decel {:?}",
        //original, heading, diff, decel
    //);

    context.pixel.all_on();
    context.pixel.render();

    context.display.clear();
    context
        .display
        .draw_text(4, 4, "Press a key...", WHITE)
        .unwrap();
    context.display.update_all().unwrap();

    while !quit {
        while let Some(event) = context.gilrs.next_event() {
            context.gilrs.update(&event);
            match event {
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::Mode, _),
                    ..
                } => {
                    println!("Mode Pressed");
                    quit = true;
                    running = false;
                    break;
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::North, _),
                    ..
                } => {
                    direction = "North";
                    running = true;
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::South, _),
                    ..
                } => {
                    direction = "South";
                    running = true;
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::West, _),
                    ..
                } => {
                    direction = "West";
                    running = true;
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::East, _),
                    ..
                } => {
                    direction = "East";
                    running = true;
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::Start, _),
                    ..
                } => {
                    direction = "Align";
                    running = true;
                }
                _ => {
                    // Swallow event
                }
            };
        }

        if running {
            //println!("Direction {:?}",direction);
            let front_distance = get_distance(&mut front, true);
            let left_distance = get_distance(&mut leftfront, true);
            let right_distance = get_distance(&mut rightfront, true);
            let back_distance = get_distance(&mut back, true);
            let mut distance = 0 as u16;
            //heading = compass.read_degrees().unwrap();
            let diff = 0;
            

            if direction == "Align" {
                control.stop();
                context.pixel.all_off();
                context.pixel.render();
                //align(original, &mut compass, &mut control, 4);
                direction = "None"
            }
            if direction == "North" {
                let bias = 40;
                distance = front_distance;
                decel = get_deceleration(distance, DIST);
                left_rear_speed = SPEED + bias;
                right_rear_speed = SPEED * -1;
                left_front_speed = SPEED + bias;
                right_front_speed = SPEED * -1;
            }
            if direction == "South" {
                let bias = 40;
                distance = back_distance;
                decel = get_deceleration(distance, DIST);
                left_rear_speed = (SPEED + bias) * -1;
                right_rear_speed = SPEED;
                left_front_speed = (SPEED + bias) * -1;
                right_front_speed = SPEED;
            }
            if direction == "West" {
                let bias = 10;
                distance = left_distance;
                //if distance > DIST {
                //distance = distance - DIST;
                //}
                decel = get_deceleration(distance, DIST);
                left_front_speed = SPEED;
                left_rear_speed = (SPEED + bias) * -1;
                right_front_speed = SPEED;
                right_rear_speed = (SPEED + bias) * -1;
            }
            if direction == "East" {
                let bias = 30;
                distance = right_distance;
                decel = get_deceleration(distance, DIST);
                left_front_speed = SPEED * -1;
                left_rear_speed = SPEED - bias;
                right_front_speed = SPEED * -1;
                right_rear_speed = SPEED - bias;
            }

            if direction == "North"
                || direction == "South"
                || direction == "West"
                || direction == "East"
            {
                left_rear_speed = ((left_rear_speed as f64) * decel) as i32;
                right_rear_speed = ((right_rear_speed as f64) * decel) as i32;
                left_front_speed = ((left_front_speed as f64) * decel) as i32;
                right_front_speed = ((right_front_speed as f64) * decel) as i32;

                control.set_gear(gear);
                control.set_bias(0);
                control.speed(
                    left_rear_speed,
                    right_rear_speed,
                    left_front_speed,
                    right_front_speed,
                );

                //println!(
                    //"Direction {:?} Org {:#?}° Head {:#?}° Decel {:#?},Dist {:?}",
                    //direction, original, heading, decel, distance
                //);

                println!(
                    "Speeds lf {:?}, lr {:#?}, rf {:#?}, rr {:#?}",
                    left_front_speed, left_rear_speed, right_front_speed, right_rear_speed
                );

                if distance < MINDIST {
                    control.stop();
                    running = false;
                    println!("Done");
                }
            }
        }
    }
    control.stop();
    context.pixel.all_off();
    context.pixel.render();
}

//use std::sync::Arc;
//use std::sync::Mutex;

fn _do_calibrate(context: &mut Context) {
    context.pixel.all_off();
    context.pixel.render();

    let (command_tx, command_rx) = mpsc::channel();
    let (data_tx, data_rx) = mpsc::channel();

    let t = thread::spawn(move || {
        println!("Thread Starting");
        let colour: i32 = 0;
        let mut cam = build_camera();
        load_calibration(&mut cam);
        loop {
            cam.discard_video();
            println!("Thread running");
            match command_rx.try_recv() {
                Ok("X") | Err(TryRecvError::Disconnected) => {
                    println!("Terminating.");
                    break;
                }
                Ok("F") => {
                    let colour = cam.get_colour(false);
                    if colour == RED {
                        let _ = data_tx.send("0");
                    }
                    if colour == BLUE {
                        let _ = data_tx.send("1");
                    }
                    if colour == YELLOW {
                        let _ = data_tx.send("2");
                    }
                    if colour == GREEN {
                        let _ = data_tx.send("3");
                    }
                    println!("Colour {:?}", colour);
                }
                Ok(&_) | Err(TryRecvError::Empty) => {}
            }
        }
    });

    let mut quit = false;
    while !quit {
        match data_rx.try_recv() {
            Ok("0") => {
                print_colour(context, RED);
            }
            Ok("1") => {
                print_colour(context, BLUE);
            }
            Ok("2") => {
                print_colour(context, YELLOW);
            }
            Ok("3") => {
                print_colour(context, GREEN);
            }
            Ok(_) | Err(_) => {}
        }

        while let Some(event) = context.gilrs.next_event() {
            context.gilrs.update(&event);
            match event {
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::Mode, _),
                    ..
                } => {
                    //println!("Mode Pressed");
                    {
                        let _ = command_tx.send("X");
                    }
                    quit = true;
                    break;
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::Start, _),
                    ..
                } => {
                    //println!("Mode Pressed");
                    {
                        let _ = command_tx.send("F");
                    }
                    break;
                }
                _ => {
                    break;
                }
            }
        }
    }
    context.pixel.all_off();
    context.pixel.render();
}

fn show_menu(context: &mut Context, menu: i8) {
    context.display.clear();
    context.display.draw_text(20, 42, "Forest", WHITE).unwrap();
    context
        .display
        .draw_text(20, 50, "Fighters", WHITE)
        .unwrap();
    context
        .display
        .draw_text(20, 58, "Ready...", WHITE)
        .unwrap();
    context.display.update_all().unwrap();

    context.display.clear();
    context
        .display
        .draw_text(4, 4, "Forest Fighters", LT_GREY)
        .unwrap();

    if menu == 0 {
        let tiny = image::open("The Canyons of Mars Menu Item.jpg").unwrap();

        context.display.draw_image(0, 16, tiny).unwrap();
        context
            .display
            .draw_text(4, 108, "Canyons of Mars", WHITE)
            .unwrap();
    } else if menu == 1 {
        let tiny = image::open("Hubble Telescope Item Menu.jpg").unwrap();

        context.display.draw_image(0, 16, tiny).unwrap();
        context
            .display
            .draw_text(12, 108, "Hubble T'scope", WHITE)
            .unwrap();
    } else if menu == 2 {
        let tiny = image::open("Blast Off Menu Item.jpg").unwrap();

        context.display.draw_image(0, 16, tiny).unwrap();
        context
            .display
            .draw_text(40, 108, "Blast Off", WHITE)
            .unwrap();
    } else if menu == 3 {
        let tiny = image::open("Large Wheels Menu Item.jpg").unwrap();

        context.display.draw_image(0, 16, tiny).unwrap();
        context
            .display
            .draw_text(4, 108, "Large Wheels RC", WHITE)
            .unwrap();
    } else if menu == 4 {
        let tiny = image::open("Mecanum Wheels Menu Item.jpg").unwrap();

        context.display.draw_image(0, 16, tiny).unwrap();
        context
            .display
            .draw_text(28, 108, "Mecanum RC", WHITE)
            .unwrap();
    } else if menu == 5 {
        let tiny = image::open("Exit Menu Item.jpg").unwrap();

        context.display.draw_image(0, 16, tiny).unwrap();
        context.display.draw_text(56, 108, "EXIT", WHITE).unwrap();
    } else if menu == 6 {
        let tiny = image::open("Shutdown Menu Item.jpg").unwrap();

        context.display.draw_image(0, 16, tiny).unwrap();
        context
            .display
            .draw_text(32, 108, "SHUTDOWN", WHITE)
            .unwrap();
    } else if menu == 7 {
        let tiny = image::open("RunTests.jpg").unwrap();

        context.display.draw_image(0, 16, tiny).unwrap();
        context
            .display
            .draw_text(32, 108, "Run Tests", WHITE)
            .unwrap();
    } else if menu == 8 {
        let tiny = image::open("Calibrate.jpg").unwrap();

        context.display.draw_image(0, 16, tiny).unwrap();
        context
            .display
            .draw_text(32, 108, "Calibrate", WHITE)
            .unwrap();
    }

    context.display.update_all().unwrap();
}

fn main() {
    // Uncomment to test
    //_test();    // sensors
    //_test2();   // camera
    //_test3();   // pixels
    //_test4();     // display
    //_test5();     // Motors
    //return;

    // A list of locations, colours are updated when found.
    let locations = [0.0, 0.0, 0.0, 0.0];
    let order = [NONE, NONE, NONE, NONE];

    //let mut pixel = build_pixel();
    //let mut gilrs = Gilrs::new().unwrap();
    //let mut display = SSD1327::new("/dev/i2c-3");

    let mut context = Context::new("/dev/i2c-3");

    context.display.begin().unwrap();

    context.display.clear();
    context.display.draw_text(20, 42, "Forest", WHITE).unwrap();
    context
        .display
        .draw_text(20, 50, "Fighters", WHITE)
        .unwrap();
    context
        .display
        .draw_text(20, 58, "Ready...", WHITE)
        .unwrap();
    context.display.update_all().unwrap();

    let mut menu: i8 = 0;
    let mut prev: i8 = -1;

    let mut quit = false;
    while !quit {
        if menu > 8 {
            menu = 0;
        } else if menu < 0 {
            menu = 8;
        }

        if menu != prev {
            prev = menu;
            show_menu(&mut context, menu);
        }

        while let Some(event) = context.gilrs.next_event() {
            match event {
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::DPadRight, _),
                    ..
                } => {
                    menu = menu + 1;
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::DPadLeft, _),
                    ..
                } => {
                    menu = menu - 1;
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::Select, _),
                    ..
                } => {
                    if menu == 0 {
                        context.display.clear();
                        context
                            .display
                            .draw_text(4, 4, "Canyon...", LT_GREY)
                            .unwrap();
                        context.display.update_all().unwrap();
                        do_canyon(&mut context);
                        prev = -1;
                    }
                    if menu == 1 {
                        context.display.clear();
                        context
                            .display
                            .draw_text(4, 4, "Hubble...", LT_GREY)
                            .unwrap();
                        context.display.update_all().unwrap();
                        do_hubble(&mut context, locations, order);
                        prev = -1;
                    }
                    if menu == 2 {
                        context.display.clear();
                        context
                            .display
                            .draw_text(4, 4, "Blast Off...", LT_GREY)
                            .unwrap();
                        context.display.update_all().unwrap();
                        do_straight(&mut context);
                        prev = -1;
                    }
                    if menu == 3 {
                        context.display.clear();
                        context
                            .display
                            .draw_text(4, 4, "Wheels RC...", LT_GREY)
                            .unwrap();
                        context.display.update_all().unwrap();
                        do_wheels_rc(&mut context);
                        prev = -1;
                    }
                    if menu == 4 {
                        context.display.clear();
                        context
                            .display
                            .draw_text(4, 4, "Mecanum RC...", LT_GREY)
                            .unwrap();
                        context.display.update_all().unwrap();
                        do_mecanum_rc(&mut context);
                        prev = -1;
                    }
                    if menu == 5 {
                        context.display.clear();
                        context
                            .display
                            .draw_text(4, 4, "Exiting...", LT_GREY)
                            .unwrap();
                        context.display.update_all().unwrap();
                        quit = true;
                        break;
                    }
                    if menu == 6 {
                        context.display.clear();
                        context
                            .display
                            .draw_text(4, 4, "Shutdown...", LT_GREY)
                            .unwrap();
                        context.display.update_all().unwrap();
                        Command::new("halt").spawn().expect("halt command failed");
                        quit = true;
                        break;
                    }
                    if menu == 7 {
                        context.display.clear();
                        context
                            .display
                            .draw_text(4, 4, "Run Tests...", LT_GREY)
                            .unwrap();
                        context.display.update_all().unwrap();
                        do_run_tests(&mut context);
                        prev = -1;
                        break;
                    }
                    if menu == 8 {
                        context.display.clear();
                        context
                            .display
                            .draw_text(4, 4, "Calibrate", LT_GREY)
                            .unwrap();
                        context.display.update_all().unwrap();
                        do_calibrate(&mut context);
                        prev = -1;
                        break;
                    }
                }
                _ => (),
            };
        }
    }

    context.display.clear();
    context.display.update_all().unwrap();
    thread::sleep(time::Duration::from_millis(2000));
}
