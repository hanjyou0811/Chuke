use crate::bootrom::Bootrom;
use crate::hram::HRam;
use crate::wram::WRam;
use crate::ppu::Ppu;
use crate::cartridge::{Cartridge};
use crate::cpu::{
    interrupts::Interrupts,
};
use crate::timer::Timer;
use crate::joypad::{Joypad};
use crate::apu::Apu;
use crate::audio::Audio;
use std::io;

pub struct Bus {
    bootrom: Bootrom,
    key0: u8,
    wram: WRam,
    hram: HRam,
    cartridge: Cartridge,
    pub apu: Apu,
    pub timer: Timer,
    pub joypad: Joypad,
    pub ppu: Ppu,
}
impl Bus {
    pub fn new(bootrom: Bootrom, cartridge: Cartridge, audio: Audio) -> Self {
        let is_cgb = bootrom.is_cgb();
        Self {
            bootrom,
            key0: 0,
            wram: WRam::new(),
            hram: HRam::new(),
            cartridge,
            ppu: Ppu::new(is_cgb),
            timer: Timer::default(),
            joypad: Joypad::new(),
            apu: Apu::new(audio),
        }
    }
    fn cgb_enabled(&self) -> bool {
        self.bootrom.is_cgb() && (self.bootrom.is_active() || self.key0 & 4 == 0)
    }

    fn is_cgb_register(addr: u16) -> bool {
        matches!(addr, 0xFF4D | 0xFF4F | 0xFF51..=0xFF56 | 0xFF68..=0xFF6C | 0xFF70)
    }

    pub fn read(&self, interrupts: &Interrupts, addr: u16) -> u8 {
        if self.bootrom.maps(addr) {
            return self.bootrom.read(addr);
        }
        if Self::is_cgb_register(addr) && !self.cgb_enabled() {
            return 0xFF;
        }
        match addr {
            0x0000..=0x7FFF => self.cartridge.read(addr),
            0x8000..=0x9FFF => self.ppu.read(addr),
            0xA000..=0xBFFF => self.cartridge.read(addr),
            0xFE00..=0xFE9F => self.ppu.read(addr),
            0xFF00          => self.joypad.read(),
            0xFF04..=0xFF07 => self.timer.read(addr),
            0xFF10..=0xFF26 | 0xFF30..=0xFF3F => self.apu.read(addr),
            0xFF40..=0xFF4B | 0xFF4F | 0xFF68..=0xFF6B => self.ppu.read(addr),
            0xFF0F          => interrupts.read(addr),
            0xFFFF          => interrupts.read(addr),
            0xC000..=0xFDFF | 0xFF70 => self.wram.read(addr),
            0xFF80..=0xFFFE => self.hram.read(addr),
            _               => 0xFF,
        }
    }
    pub fn write(&mut self, interrupts: &mut Interrupts, addr: u16, val: u8) {
        if Self::is_cgb_register(addr) && !self.cgb_enabled() {
            return;
        }
        match addr {
            0x0000..=0x7FFF => self.cartridge.write(addr, val),
            0x8000..=0x9FFF => self.ppu.write(addr, val),
            0xA000..=0xBFFF => self.cartridge.write(addr, val),
            0xFE00..=0xFE9F => self.ppu.write(addr, val),
            0xFF00          => self.joypad.write(addr, val),
            0xFF04..=0xFF07 => self.timer.write(addr, val),
            0xFF10..=0xFF26 | 0xFF30..=0xFF3F => self.apu.write(addr, val),
            0xFF40..=0xFF4B | 0xFF4F | 0xFF68..=0xFF6B  => self.ppu.write(addr, val),
            0xFF0F          => interrupts.write(addr, val),
            0xFFFF           => interrupts.write(addr, val),
            0xC000..=0xFDFF | 0xFF70 => self.wram.write(addr, val),
            0xFF4C if self.bootrom.is_cgb() && self.bootrom.is_active() => {
                self.key0 = val;
            },
            0xFF50 => {
                self.bootrom.write(addr, val);
                self.ppu.set_mode(self.cgb_enabled());
            },
            0xFF80..=0xFFFE => self.hram.write(addr, val),
            _               => (),
        }
    }

    pub fn save_cartridge_if_ready(&mut self, current_frame: u64) -> io::Result<()> {
        self.cartridge.save_if_ready(current_frame)
    }

    pub fn flush_cartridge_save(&mut self, current_frame: u64) -> io::Result<()> {
        self.cartridge.flush_save(current_frame)
    }
}
