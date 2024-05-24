mod frame_catcher;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use frame_catcher::FrameCatcher;

use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH:u32 = 320;
const HEIGHT:u32 = 224;

fn main() -> Result<(), Error> {

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new((WIDTH*2) as f64, (HEIGHT*2) as f64);
        WindowBuilder::new()
            .with_title("SEGA RGBlast")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let init_vec:Vec<u8> = vec![0xFF;pixels.get_frame_mut().len()];
    pixels.get_frame_mut().copy_from_slice(&init_vec[..]);

    let mut frame_catcher = FrameCatcher::new();
    frame_catcher.catch_single_frame(); // prime it
    // frame_catcher.start();

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let now = Instant::now();
        while now.elapsed().as_millis() <= 0 { }
        let mut frames:u128 = 0;
        loop {
            let next_frame = frame_catcher.catch_single_frame();
            frames+=1;
            if frames % 30 == 0 {
                println!("fps: {}", (((frames*10000)/now.elapsed().as_millis()) as f64) / 10.0);
            }
            tx.send(next_frame).expect("Couldn't send frame.");
        }
    });

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        let image_buffer;
        if let Event::RedrawRequested(_) = event {
            let frame_check = rx.try_recv();
            match frame_check {
                Ok(img) => {
                    image_buffer = img;
                },
                Err(_) => { return; }
            }
            let mut image_words:Vec<u32> = Vec::new();

            unsafe {
                let (prefix, words, suffix) = image_buffer.align_to::<u32>();
            
                if prefix.len() + suffix.len() >= 4 {
                    panic!("Too many bytes abandoned!?");
                }

                image_words.extend_from_slice(&words);
            }

            decode_6bpp_frame(&image_words[..], pixels.get_frame_mut());
            // println!("decoded_buffer is {} words", decoded_buffer.len());

            if let Err(err) = pixels.render() {
                *control_flow = ControlFlow::Exit;
                panic!("pixels.render() failed {:?}", err);
            }
        }
        
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() || input.destroyed() {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        window.request_redraw();
    });

}

fn decode_6bpp_frame(frame_words:&[u32], image_out:&mut[u8]) {

    let mut bit_index:usize = 0;
    let mut channels_dumped:u32 = 0;

    let mut x;
    let mut y;

    let mut channel_val:u8;

    let mut shifter;
    let mut mask;

    while channels_dumped < (320*224*3) as u32 {
        if (bit_index%32) != 0 {
            panic!("missed a bitindex.");
        }

        let mut i = 0;
        let mut container_word_index;

        while i < 15 {
            container_word_index = bit_index/32;

            shifter = 30 - (bit_index%32);
            mask = 0b11 << shifter;

            channel_val = ((frame_words[container_word_index] & mask) >> shifter) as u8;
            channel_val *= 85; // 255 / 85 = 3

            let total_pixels = channels_dumped/3;
            y = total_pixels/320;
            x = total_pixels-(y*320);

            let pixel_index = (y*320)+x;

            image_out[((pixel_index*4)+(channels_dumped%3)) as usize] = channel_val;

            bit_index += 2;
            channels_dumped += 1;

            i += 1;
        }

        container_word_index = bit_index/32;
        shifter = 30 - (bit_index%32);
        mask = 0b11 << shifter;

        channel_val = ((frame_words[container_word_index] & mask) >> shifter) as u8;

        if channel_val != 0 {
            // panic!("Parity check fail at: {}.\nBytes: {:?}", bit_index, &frame_words[container_word_index-4..container_word_index+4]);
        }
        
        bit_index += 2;
    }
}
