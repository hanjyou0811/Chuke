use sdl2::{
    event::Event,
    keyboard::Keycode,
    Sdl,
};
use crate::bootrom::Bootrom;
use crate::bus::Bus;
use crate::cartridge::{Cartridge};
use crate::cpu::Cpu;
use crate::lcd::LCD;
use crate::joypad::{Button};
use crate::audio::Audio;

use std::{io, time};
pub const CPU_CLOCK_HZ: u128 = 4_194_304;
pub const M_CYCLE_CLOCK: u128 = 4;
// 1 mcycleあたりのナノ時間
const M_CYCLE_NANOS: u128 = M_CYCLE_CLOCK * 1_000_000_000 / CPU_CLOCK_HZ;

fn key2joy(keycode: Keycode) -> Option<Button> {
    match keycode {
        Keycode::Up => Some(Button::Up),
        Keycode::Down => Some(Button::Down),
        Keycode::Left => Some(Button::Left),
        Keycode::Right => Some(Button::Right),
        Keycode::W => Some(Button::Up),
        Keycode::S => Some(Button::Down),
        Keycode::A => Some(Button::Left),
        Keycode::D => Some(Button::Right),

        Keycode::Tab => Some(Button::Select),
        Keycode::R   => Some(Button::Start),

        Keycode::Z => Some(Button::A),
        Keycode::X => Some(Button::B),
        _ => None,
    }
}

pub struct GameBoy {
    cpu: Cpu,
    bus: Bus,
    lcd: LCD,
    sdl: Sdl,
}

impl GameBoy {
    pub fn new(bootrom: Bootrom, cartridge: Cartridge) -> Self {
        let sdl = sdl2::init().expect("failed to initialize SDL");
        let lcd = LCD::new(&sdl, 4);
        let audio = Audio::new(&sdl);
        let bus = Bus::new(bootrom, cartridge, audio);
        let cpu = Cpu::new();
        Self {
            cpu, 
            bus,
            lcd,
            sdl,
        }
    }
    pub fn run(&mut self) -> io::Result<()> {
        let mut event_pump = self.sdl.event_pump().unwrap();
        let time = time::Instant::now();
        let mut elapsed = 0;
        let mut current_frame = 0u64;
        'running: loop {
            let e = time.elapsed().as_nanos();
            let mut event_cycles = 0;
            for _ in 0..(e - elapsed) / M_CYCLE_NANOS {
                event_cycles += 1;
                if event_cycles >= 1024 {
                    event_cycles = 0;
                    for event in event_pump.poll_iter() {
                        match event {
                            Event::Quit { .. } => break 'running,
                            Event::KeyDown { keycode: Some(k), .. } => {
                                if k == Keycode::Escape { break 'running }
                                key2joy(k).map(|j| self.bus.joypad.button_down(&mut self.cpu.interrupts, j));
                            },
                            Event::KeyUp { keycode: Some(k), .. } => {
                                key2joy(k).map(|j| self.bus.joypad.button_up(j));
                            },
                            _ => (),
                        }
                    }
                }
                if crate::emulation::step_base_cycle(&mut self.cpu, &mut self.bus) {
                    self.lcd.draw(self.bus.ppu.pixel_buffer());
                    current_frame = current_frame.wrapping_add(1);
                    if let Err(error) = self.bus.save_cartridge_if_ready(current_frame) {
                        eprintln!("Failed to save cartridge data: {error}");
                    }
                }
                elapsed += M_CYCLE_NANOS;
            }
        }
        self.bus.flush_cartridge_save(current_frame)
    }
}
