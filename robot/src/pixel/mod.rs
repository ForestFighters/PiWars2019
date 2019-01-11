extern crate rs_ws281x;

use self::rs_ws281x::*;

pub struct Pixel {
    pub controller: Controller
}

impl Pixel {    
    
    pub fn init( &self ){               
                
    }
        
    pub fn render( &mut self ) {
        self.controller.render().unwrap(); 
    }
    
    pub fn all_on( &mut self ) {
        let leds = self.controller.leds_mut(0);	
        leds[0] = [ 0, 0, 255, 0];
        leds[1] = [ 0, 0, 255, 0];
        leds[2] = [ 0, 0, 255, 0];
        leds[3] = [ 0, 0, 255, 0];
        leds[4] = [ 0, 0, 255, 0];
        leds[5] = [ 0, 0, 255, 0];    
            
    }   
    
    pub fn all_off( &mut self ) {
        let leds = self.controller.leds_mut(0);	
        leds[0] = [ 0, 0, 0, 0];
        leds[1] = [ 0, 0, 0, 0];
        leds[2] = [ 0, 0, 0, 0];
        leds[3] = [ 0, 0, 0, 0];
        leds[4] = [ 0, 0, 0, 0];
        leds[5] = [ 0, 0, 0, 0];            
    }   
    
    pub fn right_on( &mut self ) {
        let leds = self.controller.leds_mut(0);        
        leds[0] = [ 0, 0,   0, 0];
        leds[1] = [ 0, 0,   0, 0];
        leds[2] = [ 0, 0,   0, 0];
        leds[3] = [ 0, 0, 255, 0];
        leds[4] = [ 0, 0, 255, 0];
        leds[5] = [ 0, 0, 255, 0];           
    }

    pub fn left_on( &mut self ) {
        let leds = self.controller.leds_mut(0);        
        leds[0] = [ 0, 0, 255, 0];
        leds[1] = [ 0, 0, 255, 0];
        leds[2] = [ 0, 0, 255, 0];
        leds[3] = [ 0, 0,   0, 0];
        leds[4] = [ 0, 0,   0, 0];
        leds[5] = [ 0, 0,   0, 0];                
    }
        
}

pub fn build_pixel( ) -> Pixel {
    
    let mut controller = ControllerBuilder::new()		
        .freq(800_000)		
        .dma(10)
        .channel( 0, 
            ChannelBuilder::new()
                .pin(12)
                .count(6)
                .strip_type(StripType::Ws2812)
                .brightness(50)
                .build()
         ).build().unwrap();
         
    Pixel {
       controller      
    }
}
