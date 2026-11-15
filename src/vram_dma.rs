/// VRAM DMA addresses use a VRAM-relative destination and 16-byte blocks.
#[derive(Default)]
pub struct VramDma {
    pub(crate) src: u16,
    pub(crate) dst: u16,
    remaining: u16,
    offset: u16,
    running: bool,
    hblank: bool,
    cancelled: bool,
}

impl VramDma {
    pub fn read(&self) -> u8 {
        if self.running { (self.remaining - 1) as u8 }
        else if self.cancelled { 0x80 | (self.remaining - 1) as u8 }
        else { 0xFF }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF51 if !self.running => self.src = (self.src & 0xF0) | ((val as u16) << 8),
            0xFF52 if !self.running => self.src = (self.src & 0xFF00) | (val as u16 & 0xF0),
            0xFF53 if !self.running => self.dst = (self.dst & 0xF0) | ((val as u16 & 0x1F) << 8),
            0xFF54 if !self.running => self.dst = (self.dst & 0x1F00) | (val as u16 & 0xF0),
            0xFF55 if !self.running => {
                self.remaining = (val & 0x7F) as u16 + 1;
                self.offset = 0;
                self.hblank = val & 0x80 != 0;
                self.running = self.dst < 0x2000;
                self.cancelled = false;
            }
            _ => (),
        }
    }

    pub fn active(&self) -> bool { self.running }

    pub fn next_pair(&self) -> Option<(u16, u16)> {
        (self.running && !self.hblank).then(|| (self.src.wrapping_add(self.offset), self.dst + self.offset))
    }

    pub fn advance_pair(&mut self) {
        if !self.running { return; }
        self.offset += 2;
        if self.offset == 16 {
            self.offset = 0;
            self.src = self.src.wrapping_add(16);
            self.dst += 16;
            self.remaining -= 1;
            if self.remaining == 0 || self.dst >= 0x2000 {
                self.running = false;
            }
        }
    }
}
