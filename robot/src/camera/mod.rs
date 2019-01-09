extern crate opencv;
extern crate time;

use std::time::Instant;

use self::opencv::core;
use self::opencv::highgui;
use self::opencv::imgproc;

pub struct Camera {		
    pub cam: highgui::VideoCapture,
    
    pub red2_lower: core::Scalar,
	pub red2_upper: core::Scalar,
	
	pub red_lower: core::Scalar,
	pub red_upper: core::Scalar,
		
	pub green_lower: core::Scalar,
	pub green_upper: core::Scalar,
	
	pub blue_lower: core::Scalar,
	pub blue_upper: core::Scalar,
		
	pub yellow_lower: core::Scalar,
	pub yellow_upper: core::Scalar,
	
	pub red2: i32,
	pub red: i32,
	pub green: i32,
	pub blue: i32,
	pub yellow: i32,
	pub colours: [ i32; 5], 
}

impl Camera {    
    
    pub fn init( &mut self ){               
                
    }
    
    pub fn get_frame( &mut self ) -> core::Mat {
        let mut frame = core::Mat::new().unwrap();
		self.cam.read(&mut frame).unwrap();
		return frame;
    }  
    
    pub fn get_colour(&mut self, mut frame: core::Mat) -> Result<i32,String> {
	
		let now = Instant::now();
		println!("Start {:#?}",Instant::now().duration_since(now));
	
		let mut ret = -1;	
		
		let window = "Video Capture";
		try!(highgui::named_window(window,1));
		
		let window2 = "Overlay";
		try!(highgui::named_window(window2,1));
	    
		println!("Now {:#?}",Instant::now().duration_since(now));
    
		if try!(frame.size()).width == 0 {
			println!("Failed to create camera frame");
			let ret =-999;
			return Ok(ret);
		}
		
		println!("Now {:#?}",Instant::now().duration_since(now));
			
		let mut frame2 = try!(core::Mat::rect( &frame, core::Rect{x:0,y:200,width:640,height:80}) );
						
		let mut img_hsv = try!(core::Mat::new());
		try!(imgproc::cvt_color(&mut frame, &mut img_hsv, imgproc::COLOR_BGR2HSV, 0));
		
		let mut img_thresholded = try!(core::Mat::new());
		
		println!("Now {:#?}",Instant::now().duration_since(now));				
		
		for colour in self.colours.iter()
		{
			let mut _img_final = try!(core::Mat::new());  
			if *colour == self.red2 {
				let img_lower = try!(core::Mat::new_size_with_default( try!(img_hsv.size()), try!(img_hsv.typ()), self.red2_lower ));
				let img_upper = try!(core::Mat::new_size_with_default( try!(img_hsv.size()), try!(img_hsv.typ()), self.red2_upper ));
				try!(core::in_range( &mut img_hsv, &img_lower, &img_upper, &mut img_thresholded));      
			}
			if *colour == self.red {
				let img_lower = try!(core::Mat::new_size_with_default( try!(img_hsv.size()), try!(img_hsv.typ()), self.red_lower ));
				let img_upper = try!(core::Mat::new_size_with_default( try!(img_hsv.size()), try!(img_hsv.typ()), self.red_upper ));
				try!(core::in_range( &mut img_hsv, &img_lower, &img_upper, &mut img_thresholded));      
			}
			else if *colour == self.green {
				let img_lower = try!(core::Mat::new_size_with_default( try!(img_hsv.size()), try!(img_hsv.typ()), self.green_lower ));
				let img_upper = try!(core::Mat::new_size_with_default( try!(img_hsv.size()), try!(img_hsv.typ()), self.green_upper ));            
				try!(core::in_range( &mut img_hsv, &img_lower, &img_upper, &mut img_thresholded));
			}
			else if *colour == self.blue {	
				let img_lower = try!(core::Mat::new_size_with_default( try!(img_hsv.size()), try!(img_hsv.typ()), self.blue_lower ));
				let img_upper = try!(core::Mat::new_size_with_default( try!(img_hsv.size()), try!(img_hsv.typ()), self.blue_upper ));            
				try!(core::in_range( &mut img_hsv, &img_lower, &img_upper, &mut img_thresholded));
			}
			else if *colour == self.yellow {	
				let img_lower = try!(core::Mat::new_size_with_default( try!(img_hsv.size()), try!(img_hsv.typ()), self.yellow_lower ));
				let img_upper = try!(core::Mat::new_size_with_default( try!(img_hsv.size()), try!(img_hsv.typ()), self.yellow_upper ));            
				try!(core::in_range( &mut img_hsv, &img_lower, &img_upper, &mut img_thresholded));      
			}		
						
			let kernel = try!(imgproc::get_structuring_element(imgproc::MORPH_ELLIPSE, core::Size{width:5, height:5}, core::Point{x:-1, y:-1}));   
			let border_value = try!(imgproc::morphology_default_border_value());
						
			let mut img_dilated = try!(core::Mat::new());            
			let mut img_eroded = try!(core::Mat::new()); 
			let mut img_final = try!(core::Mat::new());  
			
			//morphological opening (removes small objects from the foreground)            
			try!(imgproc::erode( &mut img_thresholded, &mut img_eroded, &kernel, core::Point{x:-1, y:-1}, 1, imgproc::BORDER_CONSTANT, border_value));			          
			try!(imgproc::dilate( &mut img_eroded, &mut img_dilated, &kernel, core::Point{x:-1, y:-1}, 1, imgproc::BORDER_CONSTANT, border_value)); 
			
			//morphological closing (removes small holes from the foreground)
			try!(imgproc::dilate( &mut img_dilated, &mut img_eroded, &kernel, core::Point{x:-1, y:-1}, 1, imgproc::BORDER_CONSTANT, border_value)); 
			try!(imgproc::erode( &mut img_eroded, &mut img_final, &kernel, core::Point{x:-1, y:-1}, 1, imgproc::BORDER_CONSTANT, border_value)); 
			
			let result = imgproc::moments(&mut img_final, false);
			assert!( result.is_ok() );
			
			let moments = result.unwrap();		
			let area = 	moments.m00;
			//println!("Area {:#?}",area);
			if area > 5000f64
			{
				try!(highgui::imshow(window2, &mut img_final));	
				if *colour == self.red || *colour == self.red2 {
					try!(core::rectangle(&mut frame2,core::Rect{x:0,y:0,width:30,height:30},core::Scalar{ data:[0f64,0f64,255f64,-1f64] },-1 ,8 ,0));				
					ret = self.red;	
					break;						
				}
				else if *colour == self.green {
					try!(core::rectangle(&mut frame2,core::Rect{x:0,y:0,width:30,height:30},core::Scalar{ data:[0f64,255f64,0f64,-1f64] },-1 ,8 ,0));				
					ret = self.green;				
					break;
				}
				else if *colour == self.blue {	
					try!(core::rectangle(&mut frame2,core::Rect{x:0,y:0,width:30,height:30},core::Scalar{ data:[255f64,0f64,0f64,-1f64] },-1 ,8 ,0));				
					ret = self.blue;				
					break;
				}
				else if *colour == self.yellow {	
					try!(core::rectangle(&mut frame2,core::Rect{x:0,y:0,width:30,height:30},core::Scalar{ data:[0f64,255f64,255f64,-1f64] },-1 ,8 ,0));				
					ret = self.yellow;
					break;
				}
						
			}		
		}
		try!(highgui::imshow(window, &mut frame2));
		try!(highgui::wait_key(5));
		
		println!("Now {:#?}",Instant::now().duration_since(now));
		
		Ok(ret)
	} 
        
}

pub fn build_camera( ) -> Camera {
		
	let cam = highgui::VideoCapture::device(0).unwrap(); 		
	let red2 = 0;
	let red = 1;
	let green = 2;
	let blue = 3;
	let yellow = 4;
	let colours = [ red2, red, green, blue, yellow ];
	
	let red2_lower = core::Scalar{ data:[0f64,158f64,158f64,-1f64] };	
	let red2_upper = core::Scalar{ data:[10f64,255f64,255f64,-1f64] };
		
	let red_lower = core::Scalar{ data:[150f64,128f64,0f64,-1f64] };	
	let red_upper = core::Scalar{ data:[230f64,255f64,255f64,-1f64] };
			
	let green_lower = core::Scalar{ data:[55f64,60f64,91f64,-1f64] };	
	let green_upper = core::Scalar{ data:[96f64,192f64,255f64,-1f64] };
		
	let blue_lower = core::Scalar{ data:[75f64,127f64,127f64,-1f64] };	
	let blue_upper = core::Scalar{ data:[107f64,255f64,255f64,-1f64] };
			
	let yellow_lower = core::Scalar{ data:[10f64,170f64,150f64,-1f64] };	
	let yellow_upper = core::Scalar{ data:[49f64,255f64,255f64,-1f64] }; 
			
    Camera {        
        cam,
        red2_lower,
		red2_upper,	
		red_lower,
		red_upper,		
		green_lower,
		green_upper,	
		blue_lower,
		blue_upper,		
		yellow_lower,
		yellow_upper,
        red2,
		red,
		green,
		blue,
		yellow,
		colours,      
    }
}
