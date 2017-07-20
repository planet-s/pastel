//canvas widget based on image widget

use image;
use orbtk;
use orbclient::{Color, Renderer};
use orbimage;
use std::cell::{Cell, RefCell};
use std::path::Path;
//use std::fs::File;
use std::sync::Arc;
use orbtk::event::Event;
use orbtk::point::Point;
use orbtk::rect::Rect;
use orbtk::traits::{Click, Place};
use orbtk::widgets::Widget;
use std::slice;
use std::io::Error;



pub struct Canvas {
    pub rect: Cell<Rect>,
    pub image: RefCell<orbimage::Image>,
    click_callback: RefCell<Option<Arc<Fn(&Canvas, Point)>>>,
    right_click_callback: RefCell<Option<Arc<Fn(&Canvas, Point)>>>,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Arc<Self> {
        Self::from_image(orbimage::Image::new(width, height))
    }

    pub fn from_color(width: u32, height: u32, color: Color) -> Arc<Self> {
        Self::from_image(orbimage::Image::from_color(width, height, color))
    }

    pub fn from_image(image: orbimage::Image) -> Arc<Self> {
        Arc::new(Canvas {
            rect: Cell::new(Rect::new(0, 0, image.width(), image.height())),
            image: RefCell::new(image),
            click_callback: RefCell::new(None),
            right_click_callback: RefCell::new(None)
        })
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Arc<Self>, String> {
        Ok(Self::from_image(orbimage::Image::from_path(path)?))
    }
    
    pub fn save(&self, filename: &String) -> Result <i32, Error>{
        let width = self.rect.get().width as u32;
        let height = self.rect.get().height as u32;
        //get image data in form of [Color] slice
        let image_data = self.image.clone().into_inner().into_data();

        // convert u32 values to 4 * u8 (r g b a) values
        let image_buffer = unsafe {
            slice::from_raw_parts(image_data.as_ptr() as *const u8, 4 * image_data.len())
        };
        //let () = image_buffer;
        //To save corectly the image with image::save_buffer
        // we have to switch r with b but dont know why!
        let mut new_image_buffer = Vec::new();
        let mut i = 0;
        while i <= image_buffer.len() - 4 {
            new_image_buffer.push(image_buffer[i + 2]);
            new_image_buffer.push(image_buffer[i + 1]);
            new_image_buffer.push(image_buffer[i]);
            new_image_buffer.push(image_buffer[i + 3]);
            i = i + 4;
        }

        if cfg!(feature = "debug"){
            println!("Saving {}", &filename);
            println!("x{} y{} len={}", width, height, image_data.len());
        }
        
        match image::save_buffer(&Path::new(&filename),
                           &new_image_buffer,
                           width,
                           height,
                           image::RGBA(8)){
                Ok(_)   => {
                            if cfg!(feature = "debug"){println!("Saved");}
                            Ok(0)
                            },               
                Err(e) => {
                            if cfg!(feature = "debug"){println!("Error: {}",e);}
                            Err(e)
                            },
                
        }
    }

    pub fn clear(&self){
       let mut image = self.image.borrow_mut();
       //image.clear();
       image.set(Color::rgb(255, 255, 255));
    }
    
    pub fn transformation(&self, cod: &str){
        //using image::imageops library
        let width = self.rect.get().width as u32;
        let height = self.rect.get().height as u32;
        //get image data in form of [Color] slice
        let mut image_data = self.image.clone().into_inner().into_data();
        
        let image_buffer = unsafe {
            slice::from_raw_parts(image_data.as_ptr() as *const u8, 4 * image_data.len())
        };
                
        let mut imgbuf : image::ImageBuffer<image::Rgba<u8>, _> = image::ImageBuffer::from_raw(width as u32, height as u32, image_buffer.to_vec()).unwrap();
        let vec_image_buffer:Vec<u8> = image::ImageBuffer::into_raw ( match cod.as_ref() {
            
                                                             "blur"            => image::imageops::blur(&imgbuf,5.1),
                                                             "unsharpen"       => image::imageops::unsharpen(&imgbuf,5.1,10),
                                                             "flip_vertical"   => image::imageops::flip_vertical(&imgbuf),
                                                             "flip_horizontal" => image::imageops::flip_horizontal(&imgbuf),
                                                             "brighten"        => image::imageops::colorops::brighten(&imgbuf, 10),
                                                             "darken"          => image::imageops::colorops::brighten(&imgbuf, -10),
                                                                             _ => imgbuf,
         });
        
        //convert rgba 8u image buffer back into Color slice
        let mut i = 0;
        let mut r =0;
        let mut g = 0;
        let mut b =0;
        let mut a =0;
        let mut new_slice = Vec::new();
        while i <= vec_image_buffer.len() - 4 {        
            
            r = vec_image_buffer[i];
            g = vec_image_buffer[i+1];
            b = vec_image_buffer[i+2];
            a = vec_image_buffer[i+3];
            new_slice.push(orbtk::Color::rgba(b, g, r, a)); //taking care of wird bug
            i += 4;
        }
        
        let mut image = self.image.borrow_mut();
        //image.clear();
        image.image(0,0,width,height,&new_slice[..]);
        
    }
        
    
    pub fn on_right_click<T: Fn(&Self, Point) + 'static>(&self, func: T) -> &Self {
        *self.right_click_callback.borrow_mut() = Some(Arc::new(func));
        self
    }
    pub fn emit_right_click(&self, point: Point) {
        if let Some(ref right_click_callback) = *self.right_click_callback.borrow() {
            right_click_callback(self, point);
        }
    }
}

impl Click for Canvas {
    fn emit_click(&self, point: Point) {
        if let Some(ref click_callback) = *self.click_callback.borrow() {
            click_callback(self, point);
        }
    }

    fn on_click<T: Fn(&Self, Point) + 'static>(&self, func: T) -> &Self {
        *self.click_callback.borrow_mut() = Some(Arc::new(func));
        self
    }


}



impl Place for Canvas {}

impl Widget for Canvas {
    fn rect(&self) -> &Cell<Rect> {
        &self.rect
    }

    fn draw(&self, renderer: &mut Renderer, _focused: bool) {
        let rect = self.rect.get();
        let image = self.image.borrow();
        renderer.image(rect.x, rect.y, image.width(), image.height(), image.data());
    }

    fn event(&self, event: Event, focused: bool, redraw: &mut bool) -> bool {
        match event {
         /*   Event::Mouse { point, left_button, .. } => {
                let rect = self.rect.get();
                if rect.contains(point) && left_button {
                    let click_point: Point = point - rect.point();
                    self.emit_click(click_point);
                    *redraw = true;
                }
            }*/
            
            Event::Mouse {point, right_button, left_button, middle_button, ..} => {
                let rect = self.rect.get();
                if rect.contains(point) {
                    let click_point: Point = point - rect.point();
                    if right_button {
                        //println!("Right_button");
                        let click_point: Point = point - rect.point();
                        self.emit_right_click(click_point);
                        *redraw = true;
                        }
                    if left_button {
                        let click_point: Point = point - rect.point();
                        self.emit_click(click_point);
                        *redraw = true;
                        }
                    if middle_button {println!("Middle_button");}
                    }
                }
            _ => if cfg!(feature = "debug"){println!("{:?}", event)} else {()}, 
        }

        focused
    }

    fn visible(&self, flag: bool){
        !flag;
    }


}