extern crate opencv;
extern crate time;

//use std::time::Instant;

use self::opencv::core;
use self::opencv::highgui;
use self::opencv::imgproc;

pub struct Camera {
    pub cam: highgui::VideoCapture,

    //pub red2_lower: core::Scalar,
    //pub red2_upper: core::Scalar,
    pub red_lower: core::Scalar,
    pub red_upper: core::Scalar,

    pub green_lower: core::Scalar,
    pub green_upper: core::Scalar,

    pub blue_lower: core::Scalar,
    pub blue_upper: core::Scalar,

    pub yellow_lower: core::Scalar,
    pub yellow_upper: core::Scalar,

    //pub red2: i32,
    pub red: i32,
    pub green: i32,
    pub blue: i32,
    pub yellow: i32,
    pub colours: [i32; 4],
}

impl Camera {
    pub fn init(&mut self) {}

    pub fn get_colour(&mut self, visible: bool) -> i32 {
        let mat = self.get_frame();
        let colour = self.what_colour(mat, visible).unwrap();
        return colour;
    }

    pub fn search_colour(&mut self, colour_to_find: i32, visible: bool) -> bool {
        let mat = self.get_frame();
        let colour = self.what_colour(mat, false).unwrap();
        return colour == colour_to_find;
    }

    fn get_frame(&mut self) -> core::Mat {
        let mut frame = core::Mat::new().unwrap();
        self.cam.read(&mut frame).unwrap();
        return frame;
    }

    fn what_colour(&mut self, frame: core::Mat, visible: bool) -> Result<i32, String> {
        //let now = Instant::now();
        //println!("Start {:#?}",Instant::now().duration_since(now));

        let mut ret = -1;

        let window = "Video Capture";
        if visible {
            try!(highgui::named_window(window, 1));
        }

        let window2 = "Overlay";
        if visible {
            try!(highgui::named_window(window2, 1));
        }

        //println!("Now {:#?}",Instant::now().duration_since(now));

        if try!(frame.size()).width == 0 {
            println!("Failed to create camera frame");
            let ret = -999;
            return Ok(ret);
        }

        //println!("Now {:#?}",Instant::now().duration_since(now));

        let mut frame2 = try!(core::Mat::rect(
            &frame,
            core::Rect {
                x: 0,
                y: 0,
                width: 640,
                height: 80
            }
        ));

        let mut img_hsv = try!(core::Mat::new());
        //try!(imgproc::cvt_color(&mut frame2, &mut img_hsv, imgproc::COLOR_BGR2HSV, 0));
        try!(imgproc::cvt_color(
            &mut frame2,
            &mut img_hsv,
            imgproc::COLOR_BGR2YUV,
            0
        ));

        let mut img_thresholded = try!(core::Mat::new());

        //println!("Now {:#?}",Instant::now().duration_since(now));

        for colour in self.colours.iter() {
            let mut _img_final = try!(core::Mat::new());
            if *colour == self.red {
                let img_lower = try!(core::Mat::new_size_with_default(
                    try!(img_hsv.size()),
                    try!(img_hsv.typ()),
                    self.red_lower
                ));
                let img_upper = try!(core::Mat::new_size_with_default(
                    try!(img_hsv.size()),
                    try!(img_hsv.typ()),
                    self.red_upper
                ));
                try!(core::in_range(
                    &mut img_hsv,
                    &img_lower,
                    &img_upper,
                    &mut img_thresholded
                ));
            } else if *colour == self.green {
                let img_lower = try!(core::Mat::new_size_with_default(
                    try!(img_hsv.size()),
                    try!(img_hsv.typ()),
                    self.green_lower
                ));
                let img_upper = try!(core::Mat::new_size_with_default(
                    try!(img_hsv.size()),
                    try!(img_hsv.typ()),
                    self.green_upper
                ));
                try!(core::in_range(
                    &mut img_hsv,
                    &img_lower,
                    &img_upper,
                    &mut img_thresholded
                ));
            } else if *colour == self.blue {
                let img_lower = try!(core::Mat::new_size_with_default(
                    try!(img_hsv.size()),
                    try!(img_hsv.typ()),
                    self.blue_lower
                ));
                let img_upper = try!(core::Mat::new_size_with_default(
                    try!(img_hsv.size()),
                    try!(img_hsv.typ()),
                    self.blue_upper
                ));
                try!(core::in_range(
                    &mut img_hsv,
                    &img_lower,
                    &img_upper,
                    &mut img_thresholded
                ));
            } else if *colour == self.yellow {
                let img_lower = try!(core::Mat::new_size_with_default(
                    try!(img_hsv.size()),
                    try!(img_hsv.typ()),
                    self.yellow_lower
                ));
                let img_upper = try!(core::Mat::new_size_with_default(
                    try!(img_hsv.size()),
                    try!(img_hsv.typ()),
                    self.yellow_upper
                ));
                try!(core::in_range(
                    &mut img_hsv,
                    &img_lower,
                    &img_upper,
                    &mut img_thresholded
                ));
            }

            let kernel = try!(imgproc::get_structuring_element(
                imgproc::MORPH_ELLIPSE,
                core::Size {
                    width: 5,
                    height: 5
                },
                core::Point { x: -1, y: -1 }
            ));
            let border_value = try!(imgproc::morphology_default_border_value());

            let mut img_dilated = try!(core::Mat::new());
            let mut img_eroded = try!(core::Mat::new());
            let mut img_final = try!(core::Mat::new());

            //morphological opening (removes small objects from the foreground)
            try!(imgproc::erode(
                &mut img_thresholded,
                &mut img_eroded,
                &kernel,
                core::Point { x: -1, y: -1 },
                1,
                imgproc::BORDER_CONSTANT,
                border_value
            ));
            try!(imgproc::dilate(
                &mut img_eroded,
                &mut img_dilated,
                &kernel,
                core::Point { x: -1, y: -1 },
                1,
                imgproc::BORDER_CONSTANT,
                border_value
            ));

            //morphological closing (removes small holes from the foreground)
            try!(imgproc::dilate(
                &mut img_dilated,
                &mut img_eroded,
                &kernel,
                core::Point { x: -1, y: -1 },
                1,
                imgproc::BORDER_CONSTANT,
                border_value
            ));
            try!(imgproc::erode(
                &mut img_eroded,
                &mut img_final,
                &kernel,
                core::Point { x: -1, y: -1 },
                1,
                imgproc::BORDER_CONSTANT,
                border_value
            ));

            let result = imgproc::moments(&mut img_final, false);
            assert!(result.is_ok());

            if visible {
                try!(highgui::imshow(window2, &mut img_final));
            }

            let moments = result.unwrap();
            let area = moments.m00;

            //println!("Area {:#?}",area);
            if area > 5000f64 {
                if *colour == self.red {
                    try!(core::rectangle(
                        &mut frame2,
                        core::Rect {
                            x: 0,
                            y: 0,
                            width: 30,
                            height: 30
                        },
                        core::Scalar {
                            data: [0f64, 0f64, 255f64, -1f64]
                        },
                        -1,
                        8,
                        0
                    ));
                    ret = self.red;
                    break;
                } else if *colour == self.green {
                    try!(core::rectangle(
                        &mut frame2,
                        core::Rect {
                            x: 0,
                            y: 0,
                            width: 30,
                            height: 30
                        },
                        core::Scalar {
                            data: [0f64, 255f64, 0f64, -1f64]
                        },
                        -1,
                        8,
                        0
                    ));
                    ret = self.green;
                    break;
                } else if *colour == self.blue {
                    try!(core::rectangle(
                        &mut frame2,
                        core::Rect {
                            x: 0,
                            y: 0,
                            width: 30,
                            height: 30
                        },
                        core::Scalar {
                            data: [255f64, 0f64, 0f64, -1f64]
                        },
                        -1,
                        8,
                        0
                    ));
                    ret = self.blue;
                    break;
                } else if *colour == self.yellow {
                    try!(core::rectangle(
                        &mut frame2,
                        core::Rect {
                            x: 0,
                            y: 0,
                            width: 30,
                            height: 30
                        },
                        core::Scalar {
                            data: [0f64, 255f64, 255f64, -1f64]
                        },
                        -1,
                        8,
                        0
                    ));
                    ret = self.yellow;
                    break;
                }
            }
        }
        if visible {
            try!(highgui::imshow(window, &mut frame2));
        }
        try!(highgui::wait_key(5));

        //println!("Now {:#?}",Instant::now().duration_since(now));

        Ok(ret)
    }
}

pub fn build_camera() -> Camera {
    let cam = highgui::VideoCapture::device(0).unwrap();
    let red = 0;
    let blue = 1;
    let yellow = 2;
    let green = 3;
    let colours = [red, blue, yellow, green];
    
    let red_lower = core::Scalar {
        //data: [58f64, 167f64, 107f64, -1f64],
        data: [121f64, 128f64, 50f64, -1f64],
    };
    let red_upper = core::Scalar {
        //data: [199f64, 255f64, 158f64, -1f64],
        data: [188f64, 228f64, 128f64, -1f64],
    };

    let blue_lower = core::Scalar {
        //data: [49f64, 80f64, 180f64, -1f64],
        data: [203f64, 40f64, 136f64, -1f64],
    };
    let blue_upper = core::Scalar {
        //data: [89f64, 120f64, 220f64, -1f64],
        data: [251f64, 94f64, 191f64, -1f64],
    };

    let yellow_lower = core::Scalar {
        //data: [121f64, 142f64, 68f64, -1f64],
        data: [237f64, 116f64, 128f64, -1f64],
    };
    let yellow_upper = core::Scalar {
        //data: [161f64, 182f64, 108f64, -1f64],
        data: [255f64, 157f64, 163f64, -1f64],
    };

    let green_lower = core::Scalar {
        //data: [124f64, 71f64, 83f64, -1f64],
        data: [197f64, 102f64, 97f64, -1f64],
    };
    let green_upper = core::Scalar {
        //data: [164f64, 111f64, 123f64, -1f64],
        data: [243f64, 135f64, 143f64, -1f64],
    };

    Camera {
        cam,
        red_lower,
        red_upper,
        green_lower,
        green_upper,
        blue_lower,
        blue_upper,
        yellow_lower,
        yellow_upper,
        red,
        green,
        blue,
        yellow,
        colours,
    }
}
