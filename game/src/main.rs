use engine;

use spin_sleep;
use std::time;
use rand::Rng;

use log::error;
use pixels::{wgpu::Surface, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const SCREEN_HEIGHT: usize = 1080;
const SCREEN_WIDTH: usize = 1920;
const TARGET_FPS: u64 = 60;
const GAME_TITLE: &str = "Trailicca";

struct LandLocation(usize,usize);   //location type for land tile

struct LandTile {                       //a tile in the land
    spritesheet: Vec<Vec<Vec<[u8;4]>>>, //sprite sheet for tile
    frame_time: time::Duration,         //time for each frame
    frame_start: time::Instant,         //time last frame started
    frame: usize,                       //current frame index in sprite sheet
    width: usize,                       //width of sprite
    height: usize,                      //height of sprite
    coords: LandLocation,               //coordinates of tile in land
}



impl LandTile {
    ///generates new tile using stolen sprite
    fn new_america(coords: LandLocation) -> Self {          
        let spritesheet = engine::scale_spritesheet(&engine::load_spritesheet("america.gif").unwrap(), 4);
        Self {                                              //^load sprite sheet
            width: spritesheet[0][0].len(),                 //sets sprite width
            height: spritesheet[0].len(),                   //sets sprite height
            spritesheet,                                    //sets sprite sheet
            frame_time: time::Duration::from_millis(500),   //sets time of each frame
            frame_start: time::Instant::now(),              //sets time of frame start
            frame: 0,                                       //sets current sprite
            coords,                                         //sets location of tile
        }
    }
}



///generates land with stolen sprite
fn init_land(x: usize, y: usize) -> Vec<Vec<LandTile>>{
    let mut land = Vec::new();                                          //creates land var
    for yi in 0..y {                                                    //for index in row size
        land.push(Vec::new());                                          //create new row
        for xi in 0..x {                                                //for index in column size
            land[yi].push(LandTile::new_america(LandLocation(xi, yi))); //push new land tile to row,column
        }
    }
    land
}



///updates land sprite
fn update_land(land: &mut Vec<Vec<LandTile>>) {
    for row in land {                                                   //for row in land
        for tile in row {                                               //for tile in row
            if tile.frame_start.elapsed() > tile.frame_time {           //if time since frame start greater than time to display frame
                if tile.frame < tile.spritesheet.len()-1 {tile.frame+=1}//if not at last frame inc frame
                else {tile.frame = 0}                                   //else set frame to beginning
                tile.frame_start = time::Instant::now();                //set start time of frame
            }
        }
    }
}



///its called math and without it none of you would even exist
fn draw_land(screen: &mut Vec<Vec<[u8;4]>>, land: &Vec<Vec<LandTile>>) {
    let height = land[0][0].height;                                                         //get width of sprite
    let width = land[0][0].width;                                                           //get height of sprite
    let x_start = (screen[0].len()/2) - land[0].len()*(width/8);                            //set starting position of land
    let y_start = (screen.len()/2) - land.len()*(height/8);                                 //set starting position of land
    for row in land {                                                                       //for row in land
        for tile in row {                                                                   //for tile in row
            let x = x_start + (tile.coords.0*2*(width/8)) - (tile.coords.1*2*(height/8));   //let sprite x position = a lot of math
            let y = y_start + (tile.coords.1*2*(height/16)) + (tile.coords.0*2*(width/16)); //let sprite y position = a lot of math
            engine::draw_sprite(screen, &tile.spritesheet[tile.frame], (x,y))               //draw sprite at x,y
        }
    }
}



fn main() {
    let mut fps = 0;                                                                                                        //set var to record fps
    let mut frames = 0;                                                                                                     //set var to record frames this second
    let target_ft = time::Duration::from_micros(1000000 / TARGET_FPS);                                                      //set target fps
    let mut second_count = time::Instant::now();                                                                            //start second timer

    let event_loop = EventLoop::new();                                                                                      //create event loop obj
    let mut input = WinitInputHelper::new();                                                                                //create WinitIH obj
    let window = {                                                                                                          //create window obj
        let size = LogicalSize::new(SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);
        WindowBuilder::new()
            .with_title(GAME_TITLE)
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };
    let mut hidpi_factor = window.scale_factor();                                                                           //get window dimensions

    let mut rng = rand::thread_rng();                                                                                       //create random thread
    let mut land = init_land(rng.gen_range(1,6), rng.gen_range(1,6));                                                       //generate land of random size
    let mut screen = vec!(vec!([48, 47, 55, 0];SCREEN_WIDTH);SCREEN_HEIGHT);                                                //create screen

    let mut pixels = {                                                                                                      //create pixel buffer
        let surface = Surface::create(&window);
        let surface_texture = SurfaceTexture::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface);
        Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture).unwrap()
    };

    event_loop.run(move |event, _, control_flow| {                                                                          //start game loop
        let frame_time = time::Instant::now();                                                                              //set start of frame time

        if let Event::RedrawRequested(_) = event {                                                                          //if redraw requested
            engine::flatten(&screen, pixels.get_frame(), SCREEN_WIDTH);                                                     //get screen then render screen
            if pixels                                                                                                       //if rendering error
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err() {
                *control_flow = ControlFlow::Exit;                                                                          //break
                return;
            }

            frames+=1;                                                                                                      //inc frames this second
            if second_count.elapsed() > time::Duration::from_secs(1) {                                                      //if a second has elapsed
                fps = frames;                                                                                               //let fps = frames that occurred this second
                second_count = time::Instant::now();                                                                        //start new second
                frames = 0;                                                                                                 //reset frames this second to 0
            }

            if let Some(i) = (target_ft).checked_sub(frame_time.elapsed()) {                                                //if target frame time greater than this frames time
                spin_sleep::sleep(i);                                                                                       //sleep remainder
            }
        }

        if input.update(event) {                                                                                            //handle input events on loop? not just on event

            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {                                                  //if esc pressed
                *control_flow = ControlFlow::Exit;                                                                          //exit
                return;
            }

            if let Some(factor) = input.scale_factor_changed() {                                                            //if window dimensions changed
                hidpi_factor = factor;                                                                                      //update hidpi_factor
            }
            if let Some(size) = input.window_resized() {                                                                    //if window resized
                pixels.resize(size.width, size.height);                                                                     //resize pixel aspect ratio
            }
            //do world updates
            screen = vec!(vec!([48, 47, 55, 0];SCREEN_WIDTH);SCREEN_HEIGHT);                                                //flush screen screen
            update_land(&mut land);                                                                                         //update land tile sprites
            draw_land(&mut screen, &land);                                                                                  //draw land
            engine::draw_text(&mut screen, (20,40), &fps.to_string(), 32.0, [0xFF;4], engine::DEBUG_FONT);                  //draw fps debug text
            window.request_redraw();                                                                                        //request frame redraw
        }
    });
}