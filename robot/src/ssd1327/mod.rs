extern crate i2cdev;
extern crate font8x8;

use self::i2cdev::core::*;
use self::i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use self::font8x8::legacy::BASIC_LEGACY;

pub trait Display {
	fn initialize(&mut self) -> Result<(), String>;
	fn invert_display(&mut self, state: bool) -> Result<(), String>;
	fn draw_char(&mut self, x1: i16, y1: i16, chr: char, colour: u16) -> Result<(), String>;
	fn draw_text(&mut self, x1: i16, y1: i16, text: &str, colour: u16) -> Result<(), String>;
	fn draw_colour(&mut self, x: i16, y: i16, colour: u16) -> Result<(), String>;
	fn draw_pixel(&mut self, x: i16, y: i16, colour: u16) -> Result<(), String>;
	fn draw_line(&mut self, x1: i16, y1: i16, x2: i16, y2: i16, colour: u16, width: u16) -> Result<(), String>;
	fn draw_rectangle(&mut self, x1: i16, y1: i16, x2: i16, y2: i16, colour: u16, filled: bool) -> Result<(), String>;
	fn clear(&mut self) -> Result<(), String>;
	fn clear_colour(&mut self, colour: u8) -> Result<(), String>;
	fn deinitialize(&mut self) -> Result<(), String>;
	fn update(&mut self) -> Result<(), String>;
	fn update_all(&mut self) -> Result<(), String>;

	fn get_width(&self) -> u16;
	fn get_height(&self) -> u16;

	fn get_def_text_colour(&self) -> u16;
	fn get_def_bg_colour(&self) -> u16;
}

pub const BLACK: u16 = 0x00;
pub const WHITE: u16 = 0x0F;
pub const LT_GREY: u16 = 0x03;
pub const DK_GREY: u16 = 0x0C;
pub const LCD_WIDTH: u16 = 128;
pub const LCD_HEIGHT: u16 = 128;

const ADDR: u16 = 0x3D;
const COMMAND_MODE: u8 = 0x00; /* C0 and DC bit are 0 				 */
const DATA_MODE: u8 = 0x40; /* C0 bit is 0 and DC bit is 1 */

enum Command {
	InverseDisplay = 0xA7,
	NormalDisplay = 0xA6,
	Off = 0xAE,
	On = 0xAF,
	SetContrastLevel = 0x81,	
	ResumeToGDDRAM = 0xA4,

	SetDisplayOffset = 0xD3,
	SetDisplayClockDiv = 0xB3,
	SetMultiplex = 0xA8,
	SetStartLine = 0x40,
	SegRemap = 0xA0,
	
	SetColumnAddress = 0x15,
	SetRowAddress = 0x75,
	SetPhaseLength = 0xB1,
}

pub struct SSD1327 {
	lcd_width: u16,
	lcd_height: u16,    
	poled_buf: Vec<u8>,
	old_poled_buf: Vec<u8>,
	i2c: LinuxI2CDevice,
}


impl SSD1327 {
	
	pub fn new() -> SSD1327 {
		let w = LCD_WIDTH;
		let h = LCD_HEIGHT;
		SSD1327 {
			lcd_width: LCD_WIDTH,
			lcd_height: LCD_HEIGHT,            
			poled_buf: vec![0; (w/2 * h) as usize],
			old_poled_buf: vec![0; (w/2 * h) as usize],
			i2c: LinuxI2CDevice::new("/dev/i2c-9", ADDR).unwrap_or_else(|_| {
				panic!("Cannot create i2c device for the display");
			}),
		}
	}

	pub fn begin(&mut self) -> Result<(), String> {
		// initialize display        https://twitter.com/i/status/1066354613354405889
		self.send_command(Command::Off as u8);
		self.send_command(Command::SetColumnAddress as u8);
		self.send_command(0x00);
		self.send_command(0x7f);
		
		self.send_command(Command::SetRowAddress as u8);
		self.send_command(0x00);
		self.send_command(0x7f);
		
		self.send_command(Command::SetContrastLevel as u8);
		self.send_command(0x80);
		
		self.send_command(Command::SegRemap as u8);
		self.send_command(0x51);
		
		self.send_command(Command::SetStartLine as u8);
		self.send_command(0x00);
		
		self.send_command(Command::SetDisplayOffset as u8);
		self.send_command(0x00);
		
		self.send_command(Command::ResumeToGDDRAM as u8);
		self.send_command(Command::SetMultiplex as u8);
		self.send_command(0x7F);
		
		self.send_command(Command::SetPhaseLength as u8);
		self.send_command(0xF1);
		
		self.send_command(Command::SetDisplayClockDiv as u8);
		self.send_command(0x00);  //80Hz:0xc1 90Hz:0xe1   100Hz:0x00   110Hz:0x30 120Hz:0x50   130Hz:0x70 

		self.send_command(0xAB);
		self.send_command(0x01);
		
		self.send_command(0xB6);
		self.send_command(0x0F);
		
		self.send_command(0xBE);
		self.send_command(0x0F);
		
		self.send_command(0xBC);
		self.send_command(0x08);

		self.send_command(0xD5);
		self.send_command(0x62);
		
		self.send_command(0xFD);
		self.send_command(0x12);
					
		self.clear();

		self.send_command(Command::On as u8);

		Ok(())
	}

	fn send_command(&mut self, c: u8) {
		match self.i2c.smbus_write_byte_data(COMMAND_MODE, c) {
			Ok(_) => (),
			Err(x) => panic!(format!("{:?}", x)),
		};
	}

	fn send_data(&mut self, d: u8) {
		match self.i2c.smbus_write_byte_data(DATA_MODE, d) {
			Ok(_) => (),
			Err(x) => panic!(format!("{:?}", x)),
		};
	}

	pub fn clear(&mut self) {
		self.poled_buf = vec![0; (self.lcd_width /2 * self.lcd_height) as usize];
		self.old_poled_buf = self.poled_buf.clone();        
	}
	
	pub fn clear_colour(&mut self, colour: u8) {
		
		for i in 0..(self.lcd_width / 2  * self.lcd_height) {
			if i % 2 == 0 {
				self.poled_buf[i as usize] = (colour << 4) as u8| self.poled_buf[i as usize] as u8;		
			} 
			else {
				self.poled_buf[i as usize] = (colour & 0x0f) as u8| self.poled_buf[i as usize] as u8;
			}   
		}      
		self.old_poled_buf = self.poled_buf.clone();                
	}

	pub fn invert(&mut self, state: bool) {
		if state {
			self.send_command(Command::InverseDisplay as u8);
		} else {
			self.send_command(Command::NormalDisplay as u8);
		}
	}
	
	pub fn display_window(&mut self, x1: u8,y1: u8, x2: u8, y2: u8) {
		
		self.send_command(Command::SetColumnAddress as u8);
		self.send_command(x1);
		self.send_command(x2);
		
		self.send_command(Command::SetRowAddress as u8);
		self.send_command(y1);
		self.send_command(y2);		
	}

	pub fn display_all(&mut self) {
		
		let x = (self.lcd_width - 1) as u8;
		let y = (self.lcd_height - 1) as u8;
		
		self.display_window( 0 as u8, 0 as u8, x, y);

		for i in 0..(self.lcd_width / 2  * self.lcd_height) {
			let data = self.poled_buf[i as usize];
			self.send_data(data);
		}		
	}

	pub fn display(&mut self) {
		let mut first_change = 0;
		let mut last_change = self.lcd_width / 2  * self.lcd_height;
				
		let mut last_column = 0;
		let mut first_column = 127;
		for i in 0..(self.lcd_width / 2 * self.lcd_height) {			
			if self.poled_buf[i as usize] != self.old_poled_buf[i as usize] {
				if first_change == 0 {
					first_change = i;				
				}
				let current_column = (i % 64) * (2 as u16);
				if current_column < first_column {
					first_column = current_column;
				}	
				if current_column > last_column  {					
					last_column = current_column;
				}				
				last_change = i + 1;
			}
		}
				
		//println!(" First Change={}, Last Change={}",first_change, last_change);
		
		let start_column = first_column;
		let end_column = last_column;
		
		let start_row = ((first_change as f32) / 64.0).floor() as u8;				
 		let end_row = ((last_change as f32) / 64.0).floor() as u8 + 1 as u8;
						 
		self.display_window( (start_column/2) as u8, start_row as u8, (end_column/2) as u8, end_row as u8);
		
		//println!(" Start Col={}, End Col={}", start_column, end_column );
		//println!(" Start Row={}, End Row={}", start_row, end_row);
		
		
		//for row in start_row..(end_row+1) {
			//for col in start_column..(end_column+1) {
				//if col % 2 == 0 {					
					//let loc = ((col / 2) + (row as u16 * 64)) as usize; 
					////println!(" Col={}, Row={}, Loc={}",col, row, loc);
					//let data = self.poled_buf[loc];
					//if data == 0x00 { print!("  ") ; }
					//if data == 0x0F { print!(" █") ; }
					//if data == 0xF0 { print!("█ ") ; }
					//if data == 0xFF { print!("██") ; }											
					//self.send_data(data);
				//}
			//}	
			//println!("");	
		//}
		//println!("");
					
		for i in first_change..last_change {			
			let current_column = (i % 64) * (2 as u16);
			let current_row = ((i as f32) / 64.0).floor() as u8;			
			if current_column >= start_column && current_column <= end_column &&
			   current_row >= start_row && current_row <= end_row {
				let data = self.poled_buf[i as usize];	
				//if data == 0x00 { print!("  ") ; }
				//if data == 0x0F { print!(" █") ; }
				//if data == 0xF0 { print!("█ ") ; }
				//if data == 0xFF { print!("██") ; }			
				self.send_data(data);				
			}
			//if current_column == end_column {
				//println!("");	
			//}
		}		
		//println!("");

		self.old_poled_buf = self.poled_buf.clone();
		self.poled_buf = vec![0; (self.lcd_width/2 * self.lcd_height) as usize];		
	}

}

impl Display for SSD1327 {
	
	fn initialize(&mut self) -> Result<(), String> {
		self.invert(false);
		self.begin()
	}

	fn invert_display(&mut self, state: bool) -> Result<(), String> {
		self.invert(state);
		Ok(())
	}
	
	fn draw_char(&mut self, x1: i16, y1: i16, chr: char, colour: u16) -> Result<(), String> {
		
		if x1 > (self.lcd_width as i16) - 1 || y1 > (self.lcd_height as i16) - 1 || x1 < 0 || y1 < 0 {
			println!("char coords exceed the normal display range");
			return Ok(());
		}
		
		let mut x_point = x1;
		let mut y_point = y1;
		
		//println!("{}",chr);
		for x in &BASIC_LEGACY[chr as usize] {
			for bit in 0..8 {
				//match *x & 1 << bit {
					//0 => print!(" "),
					//_ => print!("█"),
				//}
				match *x & 1 << bit {
					0 => self.draw_colour( x_point, y_point , BLACK).unwrap(),
					_ => self.draw_colour( x_point, y_point , WHITE).unwrap(),
				}
				x_point = x_point + 1;
			}	
			x_point = x1;
			y_point = y_point + 1;			
			//println!(" ");
		}
		return Ok(());   		
	}
	
	fn draw_text(&mut self, x1: i16, y1: i16, text: &str, colour: u16) -> Result<(), String> {

		if x1 > (self.lcd_width as i16) - 1 || y1 > (self.lcd_height as i16) - 1 || x1 < 0 || y1 < 0 {
			println!("text coords exceed the normal display range");
			return Ok(());
		}
		
		let mut x_point = x1;
		let mut y_point = y1;

		for chr in text.chars() {
			if(x_point + 8 ) > (self.lcd_width as i16)  {
				x_point = x1;
				y_point += 8;
			}
			
			if(y_point  + 8 ) > (self.lcd_height as i16) {
				x_point = x1;
				y_point = y1;
			}
			self.draw_char(x_point, y_point, chr, colour);        
			x_point = x_point + 8;			
		}	
		return Ok(());   
	}
	
	
	fn draw_colour(&mut self, x1: i16, y1: i16, colour: u16) -> Result<(), String> {
		
		if x1 > (self.lcd_width as i16) - 1 || y1 > (self.lcd_height as i16) - 1 || x1 < 0 || y1 < 0 {
			println!("pixel coords exceed the normal display range");
			return Ok(());
		}
		
		let loc = ((x1 / 2) + (y1 * 64)) as usize;  
		
		if x1 % 2 == 0 {
			self.poled_buf[loc] = (colour << 4) as u8| self.poled_buf[loc] as u8;		
		} 
		else {
			self.poled_buf[loc] = (colour & 0x0f) as u8| self.poled_buf[loc] as u8;
		}    
		
		return Ok(());    
	}
	
	
	fn draw_pixel(&mut self, x1: i16, y1: i16, colour: u16) -> Result<(), String> {

		if x1 > (self.lcd_width as i16) - 1 || y1 > (self.lcd_height as i16) - 1 || x1 < 0 || y1 < 0 {
			println!("pixel coords exceed the normal display range");
			return Ok(());
		}

		self.draw_colour( x1, y1, colour);
		
		return Ok(());   
	}

	
	fn draw_line(&mut self, x1: i16, y1: i16, x2: i16, y2: i16, colour: u16, width: u16) -> Result<(), String> {
		if x1 > (self.lcd_width as i16) - 1 || y1 > (self.lcd_height as i16) - 1 || x1 < 0 || y1 < 0 {
			println!("start coords exceed the normal display range");
			return Ok(());
		}
		if x2 > (self.lcd_width as i16) - 1 || y2 > (self.lcd_height as i16) - 1 || x2 < 0 || y2 < 0 {
			println!("end coords exceed the normal display range");
			return Ok(());
		}
		
		let mut x_start = x1;
		let mut x_end = x2;
		if x1 > x2 {
			x_start = x2;
			x_end = x1;
		}
		
		let mut y_start = y1;
		let mut y_end = y2;
		if y1 > y2 {
			y_start = y2;
			y_end = y1;
		}
		
		
		let mut x_point = x_start;
		let mut y_point = y_start;
		let dx =x_end - x_start;						 
		let dy =y_end - y_start;
		
		let x_add = 1;
		let y_add = 1;
		
		let mut esp = dx + dy;
		
		loop {
			self.draw_pixel( x_point, y_point, colour);			
			if 2 * esp >= dy {
				if x_point == x_end {
					break;
				}
				esp += dy;
				x_point += x_add;
			}
			if 2 * esp <= dx {
				if y_point == y_end {
					break;
				}
				esp += dx;
				y_point += y_add;
			}
		}
		
		Ok(())
	}
	
	
	fn draw_rectangle(&mut self, x1: i16, y1: i16, x2: i16, y2: i16, colour: u16, filled: bool) -> Result<(), String> {
		
		if x1 > (self.lcd_width as i16) - 1 || y1 > (self.lcd_height as i16) - 1 || x1 < 0 || y1 < 0 {
			println!("top left coords exceed the normal display range");
			return Ok(());
		}
		if x2 > (self.lcd_width as i16) - 1 || y2 > (self.lcd_height as i16) - 1 || x2 < 0 || y2 < 0 {
			println!("bottom right coords exceed the normal display range");
			return Ok(());
		} 
	
		let mut x_start = x1;
		let mut x_end = x2;
		if x1 > x2 {
			x_start = x2;
			x_end = x1;
		}
		
		let mut y_start = y1;
		let mut y_end = y2;
		if y1 > y2 {
			y_start = y2;
			y_end = y1;
		}

		let mut x_point = x_start;
		let mut y_point = y_start;
		if filled {
			 for y_point in y_point..y_end {					
				self.draw_line(x_start, y_point, x_end, y_point, colour, 1);
			}
		} else {
			self.draw_line(x_start, y_start, x_end, y_start, colour, 1);
			self.draw_line(x_end, y_start, x_end, y_end, colour, 1);			
			self.draw_line(x_end, y_end, x_start, y_end, colour, 1);			
			self.draw_line(x_start, y_end, x_start, y_start, colour, 1);			
		}
		Ok(())
	}
	
	fn clear_colour(&mut self, colour: u8 ) -> Result<(), String> {
		
		println!("Clear colour");
		self.clear_colour(colour);
		Ok(())
	}

	fn clear(&mut self) -> Result<(), String> {
		
		println!("Clear");
		self.clear();		
		Ok(())
	}

	fn deinitialize(&mut self) -> Result<(), String> {
		self.send_command(Command::Off as u8);
		Ok(())
	}

	fn update(&mut self) -> Result<(), String> {
		self.display();
		Ok(())
	}
	
	fn update_all(&mut self) -> Result<(), String> {
		self.display_all();		
		Ok(())
	}

	fn get_width(&self) -> u16 {
		self.lcd_width
	}

	fn get_height(&self) -> u16 {
		self.lcd_height
	}

	fn get_def_text_colour(&self) -> u16 {
		WHITE
	}

	fn get_def_bg_colour(&self) -> u16 {
		BLACK
	}
}
