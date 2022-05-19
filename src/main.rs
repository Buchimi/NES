mod bus;
mod cartridge;
mod cpu;
mod opcodes;

use bus::*;
use cartridge::Rom;
use cpu::*;

use rand::Rng;
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    EventPump,
};
#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate bitflags;

fn main() {
    //config
    let game_name = "Snake";
    let dimensions = (32.0, 32.0);

    //sdl2
    let sdl_content = sdl2::init().expect("SDL couldn't be initialized");
    let video_subsystem = sdl_content
        .video()
        .expect("Video subsystem couldn't be initialized");

    let window = video_subsystem
        .window(
            game_name,
            (dimensions.0 * 10.0) as u32,
            (dimensions.1 * 10.0) as u32,
        )
        .build()
        .expect("The window wasn't initialized");

    //we will be drawing on this canvas
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    let mut event_pump = sdl_content.event_pump().unwrap();

    //scale up the canvas
    canvas.set_scale(10.0, 10.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(
            PixelFormatEnum::RGB24,
            dimensions.0 as u32,
            dimensions.1 as u32,
        )
        .unwrap();

    //load the game in the CPU
    let bytes: Vec<u8> = std::fs::read("snake.nes").unwrap();
    let rom = Rom::new(&bytes).unwrap();

    let bus = Bus::new(rom);
    let mut cpu = CPU::new(bus);
    cpu.reset();
    //end load program

    let mut screen_state = [0 as u8; 32 * 3 * 32];
    let mut rng = rand::thread_rng();
    //run the game cycle

    cpu.run_with_callback(move |cpu| {
        //read user input
        handle_user_input(cpu, &mut event_pump);

        //update mem[0xFE]
        cpu.mem_write(0xfe, rng.gen_range(1..16));

        //read screen data
        if read_screen_state(cpu, &mut screen_state) {
            //render screen
            texture.update(None, &screen_state, 32 * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }

        //to add delay
        ::std::thread::sleep(std::time::Duration::new(0, 70_000))
    })
}

fn handle_user_input(cpu: &mut CPU, event_pump: &mut EventPump) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => std::process::exit(0),
            //when we get a movement key, write a synonymous value to address 0xFF
            Event::KeyDown {
                keycode: Some(Keycode::W),
                ..
            } => {
                cpu.mem_write(0xff, 0x77);
            }
            Event::KeyDown {
                keycode: Some(Keycode::S),
                ..
            } => {
                cpu.mem_write(0xff, 0x73);
            }
            Event::KeyDown {
                keycode: Some(Keycode::A),
                ..
            } => {
                cpu.mem_write(0xff, 0x61);
            }
            Event::KeyDown {
                keycode: Some(Keycode::D),
                ..
            } => {
                cpu.mem_write(0xff, 0x64);
            }
            _ => { /* do nothing */ }
        }
    }
}

fn color(byte: u8) -> Color {
    match byte {
        0 => sdl2::pixels::Color::BLACK,
        1 => sdl2::pixels::Color::WHITE,
        2 | 9 => sdl2::pixels::Color::GREY,
        3 | 10 => sdl2::pixels::Color::RED,
        4 | 11 => sdl2::pixels::Color::GREEN,
        5 | 12 => sdl2::pixels::Color::BLUE,
        6 | 13 => sdl2::pixels::Color::MAGENTA,
        7 | 14 => sdl2::pixels::Color::YELLOW,
        _ => sdl2::pixels::Color::CYAN,
    }
}

fn read_screen_state(cpu: &CPU, frame: &mut [u8; 32 * 3 * 32]) -> bool {
    let mut frame_idx = 0;
    let mut update = false;
    for i in 0x0200..0x600 {
        let color_idx = cpu.mem_read(i as u16);
        let (b1, b2, b3) = color(color_idx).rgb();
        if frame[frame_idx] != b1 || frame[frame_idx + 1] != b2 || frame[frame_idx + 2] != b3 {
            frame[frame_idx] = b1;
            frame[frame_idx + 1] = b2;
            frame[frame_idx + 2] = b3;
            update = true;
        }
        frame_idx += 3;
    }
    update
}
