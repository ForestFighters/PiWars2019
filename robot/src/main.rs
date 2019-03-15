extern crate byteorder;
extern crate gilrs;
extern crate i2cdev;
extern crate image;
extern crate rust_pigpio;
extern crate serde;
extern crate serde_json;
extern crate csv;


extern crate robot;

use rust_pigpio::*;
use std::time::Duration;
use std::time::Instant;
use std::{thread, time};
use std::process::Command;
use std::fs::{self, File};
use std::str;
use csv::StringRecord;

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

const MINDIST: u16 = 250;
const MAXDIST: u16 = 2000;

const TURNING_SPEED: i32 = 600;
const DRIVING_SPEED: i32 = 600;

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
    // Test compass
    let mut compass = HMC5883L::new("/dev/i2c-1").unwrap();
    println!("Compass started");

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
        println!(
            "\x1B[HCurrent Heading {:.*}  ",
            1,
            compass.read_degrees().unwrap()
        );
        println!("Left Back Distance   {:.*}   ", 1, leftback.read());
        println!("Left Front Distance  {:.*}   ", 1, leftfront.read());
        println!("Back Distance        {:.*}   ", 1, back.read());
        println!("Front Distance       {:.*}   ", 1, front.read());
        println!(
            "Right Back Distance  {:.*}   ",
            1,
            rightback.read()
        );
        println!(
            "Right Front Distance {:.*}   ",
            1,
            rightfront.read()
        );
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

fn get_deceleration(distance: u16) -> f64 {
    if distance < MINDIST {
        return 0.2;
    }
    let distance_togo = distance - MINDIST;
	let mut decel = 1.0;
	if distance_togo < 400 {
		decel = ((distance_togo as f64)/400.0); // /2.0;
        if decel < 0.2 {
            decel = 0.2;
        }        
	}
    //println!("decel is: {:?}", decel);
	return decel;
}


const SPEED: i32 = 250;
const MULTI: f32 = 100.0;
fn _align(original: f32, compass: &mut HMC5883L, control: &mut Control, gear: i32) {    
    
    control.stop();
    let mut heading = compass.read_degrees().unwrap();
    let mut diff = ((heading - original) * MULTI) as i32;
    while diff > 5 || diff < -5 {
        heading = compass.read_degrees().unwrap();
        diff = ((heading - original) * MULTI) as i32;
        println!(
                    "Heading {:#?}°:{:#?}°  Diff {:#?}",
                     original, heading, diff
                );
        if diff < 0 {
            control.turn_right( SPEED, 4);
        } else {
            control.turn_left( SPEED, 4);  
        }
    }
    control.stop();
}
    
fn do_canyon(context: &mut Context) {
    
    let mut compass = HMC5883L::new("/dev/i2c-1").unwrap();
    
    // Distance sensors group 1
    let mut front = try_open_tof("/dev/i2c-8");
    let mut leftfront = try_open_tof("/dev/i2c-5");
    let mut rightfront = try_open_tof("/dev/i2c-10");    
    println!("front started");    
    println!("left front started");    
    println!("right front started");

    // Distance sensors group 2
    let mut back = try_open_tof("/dev/i2c-7");
    let mut leftback = try_open_tof("/dev/i2c-6");
    let mut rightback = try_open_tof("/dev/i2c-9");
    println!("back started");        
    println!("left back started");
    println!("right back started");
    
    
    set_continous(&mut front);
    set_continous(&mut leftfront);
    set_continous(&mut rightfront);
    set_continous(&mut back);
    set_continous(&mut leftback);
    set_continous(&mut rightback);    
    
    let mut control = build_control();
    control.init();

    let mut distance: u16 = 0;
    let mut direction = "Front";
    let mut prev_dir = "None";
    let mut state = 1;
    let mut diff: i32 = 0;
    
    let original = compass.read_degrees().unwrap();
    let mut heading = compass.read_degrees().unwrap();    
    
    

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
    
    println!(
        "Init Original {:#?}°,  Heading {:#?}° Diff {:#?} Decel {:?}",
        original, heading, diff, decel
    );

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
            heading = compass.read_degrees().unwrap();

            //let diff = ((heading - original) * MULTI) as i32;
            let diff: i32 = 0;

            //let front_dist = front.read();
            //let front_right_dist = rightfront.read();
            //let back_right_dist = rightback.read();
            //let front_left_dist = leftfront.read();
            //let back_left_dist = leftback.read();
            //let back_dist = back.read();
            
            let front_dist = get_distance(&mut front, true);
            let front_right_dist = get_distance(&mut rightfront, true);
            let front_left_dist = get_distance(&mut leftfront, true);
            let back_dist = get_distance(&mut back, true);

            if state == 1 {
                distance = front_right_dist;
                decel = get_deceleration( distance );
                if front_dist < MINDIST {
                    //align( original, &mut compass, &mut control, gear );
                    state = 2;
                    direction = "Left";
                    distance = front_left_dist;
                    decel = get_deceleration( distance );
                }
            }

            if state == 2 && front_left_dist < MINDIST {
                // && front_dist < 150
                //align( original, &mut compass, &mut control, gear );
                state = 3;
                direction = "Back";
                distance = front_left_dist;
                decel = get_deceleration( distance );
            }
            if state == 3 && back_dist < MINDIST {
                // && front_right_dist < 150 && front_left_dist > 750
                //align( original, &mut compass, &mut control, gear );
                state = 4;
                direction = "Left";
                distance = back_dist;
                decel = get_deceleration( distance );
            }
            if state == 4 && front_left_dist < MINDIST {
                // && back_dist < 150 && front_right_dist > 750
                //align( original, &mut compass, &mut control, gear );
                state = 5;
                direction = "Front";
                distance = front_left_dist;
                decel = get_deceleration( distance );
            }
            if state == 5 && front_dist < MINDIST {
                // && front_right_dist < 150 && front_left_dist > 750
                //align( original, &mut compass, &mut control, gear );
                state = 6;
                direction = "Left";
                distance = front_dist;
                decel = get_deceleration( distance );
            }
            if state == 6 && front_left_dist < MINDIST {
                // && front_dist < 150 && front_right_dist > 750
                //align( original, &mut compass, &mut control, gear );
                state = 7;
                direction = "Back";
                distance = front_left_dist;
                decel = get_deceleration( distance );
            }
            if state == 7 && back_dist < MINDIST {
                // && front_left_dist < 150 && front_dist > 750
                //align( original, &mut compass, &mut control, gear );
                state = 8;
                direction = "Right";
                distance = back_dist;
                decel = get_deceleration( distance );
            }
            if state == 8 && front_right_dist < MINDIST {
                // && front_left_dist > 750 && front_dist > 750
                //align( original, &mut compass, &mut control, gear );
                state = 9;
                direction = "Back";
                distance = front_right_dist;
                decel = get_deceleration( distance );
            }
            if state == 9 && front_right_dist > MINDIST && front_dist > MAXDIST {
                state = 10;
                direction = "Finished";
                distance = front_dist;
                control.stop();
            }
            if state == 10 {
                break;
            }

            //if direction != prev_dir {
                println!(
                    "State {:?}, Dir {:#?}, Dist {:#?}mm  Heading {:#?}° Diff {:#?} Decel {:?}",
                    state, direction, distance, heading, diff, decel
                );
                prev_dir = direction;
            //}

            if direction == "Front" {
                left_rear_speed = SPEED;
                right_rear_speed = (SPEED + diff) * -1;
                left_front_speed = SPEED;
                right_front_speed = (SPEED + diff) * -1;
                current_colour = GREEN;
            } else if direction == "Back" {
                left_rear_speed = (SPEED + diff) * -1;
                right_rear_speed = SPEED;
                left_front_speed = (SPEED + diff) * -1;
                right_front_speed = SPEED;
                current_colour = RED;
            } else if direction == "Right" {
                // Strafe Right                
                left_front_speed = SPEED  * -1;
                left_rear_speed = SPEED; //+ diff;
                right_front_speed = SPEED  * -1;
                right_rear_speed = SPEED; //+ diff;                
                current_colour = PURPLE;
            } else if direction == "Left" {
                // Strafe Left
                left_front_speed = SPEED; //+ diff;
                left_rear_speed = SPEED  * -1;
                right_front_speed = SPEED; // + diff;
                right_rear_speed = SPEED  * -1;
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

fn print_colour(context: &mut Context, colour: i32) {
    match colour {
        RED => {
            println!("Found Red!");
            context.pixel.red();
            context.pixel.render();
        }
        BLUE => {
            println!("Found Blue!");
            context.pixel.blue();
            context.pixel.render();
        }
        YELLOW => {
            println!("Found Yellow!");
            context.pixel.yellow();
            context.pixel.render();
        }
        GREEN => {
            println!("Found Green!");
            context.pixel.green();
            context.pixel.render();
        }
        _ => {
            println!("Found Unknown");
            context.pixel.all_off();
            context.pixel.render();
        }
    }
}

fn do_hubble(context: &mut Context, mut locations: [i32; 4]) {
    context.pixel.all_on();
    let mut control = build_control();
    control.init();
    
    let mut cam = build_camera();
    
    let mut compass = HMC5883L::new("/dev/i2c-1").unwrap();

    let mut front = VL53L0X::new("/dev/i2c-8").unwrap();

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
    
    let mut heading = compass.read_degrees().unwrap();
    let mut target = compass.read_degrees().unwrap();
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
                    heading = compass.read_degrees().unwrap();
                    target = calc_target(heading, -46f32);
                    println!("Init Heading {}", heading);
                    println!("Init Target {}", target);
                    control.turn_left(TURNING_SPEED, gear);
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
                    quit = true;
                    break;
                }
                _ => (),
            };
        }
        
        // Main State running or not, first time through && locations[0] == NONE
        if running {
            heading = compass.read_degrees().unwrap();
            println!("Heading {}", heading);
            if heading < target {
                control.stop();
                thread::sleep(time::Duration::from_millis(2000));
                let colour = cam.get_colour(false);
                print_colour(context, colour);
                target = calc_target(target, -91f32);
                control.turn_left(TURNING_SPEED, gear);
            }
        }
    }

    control.stop();
    context.display.clear();
    context.pixel.all_off();
}

fn calc_target( heading: f32, offset: f32 ) -> f32 {
    
    let mut target = heading + offset;
    if target >= 360f32 {
        target = target - 360f32;
    } else if target < 0f32 {
        // offset is -ve
        target = 360f32 + (heading + offset);
    }
    println!("Target {}", target);
    return target;
 }  
    
    
fn _do_hubble(context: &mut Context, mut locations: [i32; 4]) {
    context.pixel.all_on();

    let mut control = build_control();
    control.init();

    //let mut current = RED;
    let colours = [RED, BLUE, YELLOW, GREEN];

    let mut got_red = false;
    let mut got_blue = false;
    let mut got_yellow = false;
    let mut got_green = false;

    let mut cam = build_camera();

    let mut front = VL53L0X::new("/dev/i2c-8").unwrap();

    context.display.clear();
    context
        .display
        .draw_text(4, 4, "Press Left(E) or Right(W)...", WHITE)
        .unwrap();
    context.display.update_all().unwrap();

    let mut pos = 0;
    let mut running = false;
    let mut rotation = Rotation::StartLeft;
    let mut activity = Activities::Waiting;

    let mut current = 0;
    let mut gear = 4;
    control.set_gear(gear);
    control.set_bias(0);

    println!("Press Left(E) or Right(W)...");

    let mut quit = false;
    while !quit {
        // action items
        // 1) Clear Memory
        // 2a) Searching start left
        //		activity = Searching;
        //		startLeft = true;
        // 2b) Searching start right
        //		activity = Searching;
        //		startLeft = false;
        while let Some(event) = context.gilrs.next_event() {
            match event {
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::North, _),
                    ..
                } => {
                    println!("North Pressed");
                    // Clear Memory
                    context.pixel.all_on();
                    locations = [NONE, NONE, NONE, NONE];
                    context.pixel.all_off();
                    activity = Activities::Waiting;
                }
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::West, _),
                    ..
                } => {
                    println!("West Pressed");
                    // Start button -> running
                    context.pixel.all_off();
                    context
                        .display
                        .draw_text(4, 4, "              ", WHITE)
                        .unwrap();
                    context.display.update().unwrap();
                    running = true;
                    rotation = Rotation::StartLeft;
                    activity = Activities::Searching;
                }
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
                    rotation = Rotation::StartRight;
                    activity = Activities::Searching;
                }
                // Needs gear changing here
                Event {
                    id: _,
                    event: EventType::ButtonPressed(Button::Mode, _),
                    ..
                } => {
                    println!("Mode");
                    // Mode to exit
                    quit = true;
                    break;
                }
                _ => (),
            };
        }

        // Main State running or not, first time through && locations[0] == NONE
        if running {
            // Activity State
            if activity == Activities::Searching {
                // Get the colour and store it away
                let colour = cam.get_colour(false);
                if colour == RED && !got_red {
                    print_colour(context, colour);
                    locations[pos] = RED;
                    pos += 1;
                    got_red = true;
                }
                if colour == BLUE && !got_blue {
                    print_colour(context, colour);
                    locations[pos] = BLUE;
                    pos += 1;
                    got_blue = true;
                }
                if colour == YELLOW && !got_yellow {
                    locations[pos] = YELLOW;
                    print_colour(context, colour);
                    pos += 1;
                    got_yellow = true;
                }
                if colour == GREEN && !got_green {
                    print_colour(context, colour);
                    locations[pos] = GREEN;
                    pos += 1;
                    got_green = true;
                }

                // Compute the last colour based on the first 3
                if got_red && got_blue && got_yellow && !got_green {
                    got_green = true;
                    locations[3] = GREEN;
                    activity = Activities::Finished;
                } else if got_red && got_blue && got_green && !got_yellow {
                    got_yellow = true;
                    locations[3] = YELLOW;
                    activity = Activities::Finished;
                } else if got_red && got_green && got_yellow && !got_blue {
                    got_blue = true;
                    locations[3] = BLUE;
                    activity = Activities::Finished;
                } else if got_blue && got_green && got_yellow && !got_red {
                    got_red = true;
                    locations[3] = RED;
                    activity = Activities::Finished;
                }

                // Found the colour we are looking for?	Then increament the current colour and move on
                //if colours[current] == colour {
                //println!("Current = {:?}", current);
                //print_colour( colour);
                //current += 1;
                //activity = Activities::MoveTowards;
                //} else {
                if rotation == Rotation::StartLeft {
                    control.turn_left(TURNING_SPEED, gear);
                } else {
                    control.turn_right(TURNING_SPEED, gear);
                }
                activity = Activities::Searching;
            //}
            } else if activity == Activities::MoveTowards {
                // May have to check if we are square to the target?
                let dist = front.read();
                println!("Distance = {:?}", dist);
                if dist < 130 {
                    println!("At min distance");
                    if colours[current] == GREEN {
                        activity = Activities::Done;
                    } else {
                        activity = Activities::MoveAway;
                    }
                } else {
                    control.drive(DRIVING_SPEED, gear);
                    activity = Activities::MoveTowards;
                }
            } else if activity == Activities::MoveAway {
                let dist = front.read();
                println!("Distance = {:?}", dist);
                if dist > 600 {
                    println!("At max distance");
                    activity = Activities::Complete;
                } else {
                    control.drive(DRIVING_SPEED, gear);
                    activity = Activities::MoveAway;
                }
            } else if activity == Activities::Complete {
                println!("Resume searching");
                activity = Activities::Searching;
            } else if activity == Activities::Done {
                control.stop();
                break;
            } else if activity == Activities::Finished {
                // Quit
                quit = true;
                break;
            } else if activity == Activities::Test {
                // For testing
                break;
            }
        }
    }

    control.stop();
    context.display.clear();
    context.pixel.all_off();
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

            println!("Target {:#?}mm, Right {:#?}mm, Left {:#?}mm ", target, right_dist, left_dist);

            let difference: i32 = (target - (left_dist - right_dist)) * 5;

            if difference > 15 {
                // turn right
                context.pixel.right_red();
                context.pixel.render();
                println!("Turn Right {:04}  ", difference);
                left_front_speed = left_front_speed; //+ difference;
                left_rear_speed = left_rear_speed;  //+ difference;
                right_front_speed = right_front_speed + difference;
                right_rear_speed = right_rear_speed  + difference;
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
            }  else {
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

fn get_distance(tof: &mut Option<VL53L0X>, continous: bool ) -> u16 {
    let dist: u16;
    if continous {
        match tof {
            None => dist = 0,
            Some(ref mut tof) => {
                dist = tof.read_continous();
            }
        }
    } 
    else {
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
        Some(ref mut tof) => {
            match tof.start_continuous() {
                Ok(()) => { println!("Set continuous"); },
                Err(e) => { println!("Failed to set continuous {:?}", e); }
            }   
        }
    }    
}

fn do_run_tests(context: &mut Context) {
    
    let mut cam = build_camera();
    load_calibration( &mut cam );
     
    // Test compass
    let mut compass = HMC5883L::new("/dev/i2c-1").unwrap();
    println!("Compass started");

    // Test distance sensors group 1 (not always present)
    let mut front = try_open_tof("/dev/i2c-8");
    let mut leftfront = try_open_tof("/dev/i2c-5");
    let mut rightfront = try_open_tof("/dev/i2c-10");
    
    // Test distance sensors group 2
    let mut back = try_open_tof("/dev/i2c-7");
    let mut leftback = try_open_tof("/dev/i2c-6");
    let mut rightback = try_open_tof("/dev/i2c-9");

    let mut heading = compass.read_degrees().unwrap();
    
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

    let mut colour = 0;
    context.pixel.red();
    context.pixel.render();

    let interval = Duration::from_millis(200);
    let mut now = Instant::now();

    let mut quit = false;
    while !quit {
        
        cam.discard_video();
        
        if Instant::now().duration_since(now) > interval {
            now = Instant::now();
            colour = colour + 1;
            if colour > ALL {
                colour = RED;
            }
            if colour == RED {
                context.pixel.red();
                context.pixel.render();
            } else if colour == GREEN {
                context.pixel.green();
                context.pixel.render();
            } else if colour == BLUE {
                context.pixel.blue();
                context.pixel.render();
            } else if colour == YELLOW {
                context.pixel.yellow();
                context.pixel.render();
            } else if colour == PURPLE {
                context.pixel.purple();
                context.pixel.render();
            } else if colour == CYAN {
                context.pixel.cyan();
                context.pixel.render();
            } else if colour == ALL {
                context.pixel.white();
                context.pixel.render();
            }
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
                    let colour_visible = cam.get_colour( false );
                    heading = compass.read_degrees().unwrap();
                    bk_dist = get_distance(&mut back, true);
                    lb_dist = get_distance(&mut leftback, true);
                    rb_dist = get_distance(&mut rightback, true);

                    ft_dist = get_distance(&mut front, true);
                    lf_dist = get_distance(&mut leftfront, true);
                    rf_dist = get_distance(&mut rightfront, true);

                    context.display.clear();
                    context
                        .display
                        .draw_text(0, 8, "Head:           ", WHITE)
                        .unwrap();
                    context
                        .display
                        .draw_text(56, 8, &format!("{:5.2} ", heading), WHITE)
                        .unwrap();
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
                    context
                        .display
                        .draw_text(56, 64, &format!("{:?} ", colour_visible), WHITE)
                        .unwrap();
                    context.display.update().unwrap();
                    break;
                }
                _ => {
                    break;
                }
            };
        } 
        heading = compass.read_degrees().unwrap();       
        ft_dist = get_distance(&mut front, true);
        bk_dist = get_distance(&mut back, true);
        lb_dist = get_distance(&mut leftback, true);
        rb_dist = get_distance(&mut rightback, true);
        lf_dist = get_distance(&mut leftfront, true);
        rf_dist = get_distance(&mut rightfront, true);

        println!("Current Heading      {:5.2}  ", heading);
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


fn load_calibration( cam: &mut Camera ) {
    let file = fs::File::open("calibrate.json").expect("file should open read only");
    let mut calibrate: Calibrate = serde_json::from_reader(file).expect("file should be proper JSON");  
    
    println!("Calibrate {:?}",calibrate);
    cam.set_red_lower( &mut calibrate.red_lower ); 
    cam.set_red_upper( &mut calibrate.red_upper ); 
    cam.set_green_lower( &mut calibrate.green_lower ); 
    cam.set_green_upper( &mut calibrate.green_upper ); 
    cam.set_blue_lower( &mut calibrate.blue_lower ); 
    cam.set_blue_upper( &mut calibrate.blue_upper ); 
    cam.set_yellow_lower( &mut calibrate.yellow_lower ); 
    cam.set_yellow_upper( &mut calibrate.yellow_upper ); 
    
    cam.dump_bounds();
}


fn do_calibrate(context: &mut Context) {
    
    context.pixel.all_off();
    context.pixel.render();
    
    let mut compass = HMC5883L::new("/dev/i2c-1").unwrap();    
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
    
    let original = compass.read_degrees().unwrap();
    let mut heading = compass.read_degrees().unwrap();    
    
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
    
    println!(
        "Init Original {:#?}°,  Heading {:#?}° Diff {:#?} Decel {:?}",
        original, heading, diff, decel
    );

    context.pixel.all_on();
    context.pixel.render();

    context.display.clear();
    context
        .display
        .draw_text(4, 4, "Press start...", WHITE)
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
                _ => {
                    // Swallow event
                }
            };
        }
        if running {
            println!("Direction {:?}",direction);               
            let mut distance = get_distance(&mut front, true);
            //heading = compass.read_degrees().unwrap();
            //let diff = ((heading - original) * MULTI) as i32;
            let diff = 0;
            let bias = 50;
            decel = get_deceleration( distance );
            
            if direction == "North" {
                distance = get_distance(&mut front, true);
                left_rear_speed = SPEED + bias;
                right_rear_speed = SPEED * -1;
                left_front_speed = SPEED + bias;
                right_front_speed = SPEED * -1;
            } else if direction == "South" {
                distance = get_distance(&mut back, true);
                left_rear_speed = (SPEED + bias) * -1;
                right_rear_speed = SPEED;
                left_front_speed = (SPEED + bias) * -1;
                right_front_speed = SPEED;
            } else if direction == "West" {                
                distance = get_distance(&mut leftfront, true);                
                left_front_speed = SPEED;
                left_rear_speed = SPEED  * -1;
                right_front_speed = SPEED;
                right_rear_speed = SPEED  * -1;                
            } else if direction == "East" {                
                distance = get_distance(&mut rightfront, true);
                left_front_speed = SPEED  * -1;
                left_rear_speed = SPEED; 
                right_front_speed = SPEED  * -1;
                right_rear_speed = SPEED;                
            } 
            
            left_rear_speed = ((left_rear_speed as f64) * decel) as i32;
            right_rear_speed = ((right_rear_speed as f64) * decel) as i32;
            left_front_speed = ((left_front_speed as f64) * decel) as i32;
            right_front_speed = ((right_front_speed as f64) * decel) as i32;
        
            control.speed(
                left_rear_speed,
                right_rear_speed,
                left_front_speed,
                right_front_speed,
            );
            
            println!(
                    "Heading {:#?}°,Dist {:?}  Speeds lf {:?}, lr {:#?}, rf {:#?}, rr {:#?}",
                    heading, distance, left_front_speed, left_rear_speed, right_front_speed, right_rear_speed
                );
                
            if distance < MINDIST {
                control.stop();
                running = false;
                println!("Done");
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
    }
    else if menu == 8 {
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
    let locations = [NONE, NONE, NONE, NONE];

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
                        do_hubble(&mut context, locations);
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
                        Command::new("halt")
                            .spawn()
                            .expect("halt command failed");
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
