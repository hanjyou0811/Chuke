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
    block_active: bool,
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
            0xFF55 if self.running && self.hblank && val & 0x80 == 0 => {
                self.running = false;
                self.block_active = false;
                self.cancelled = true;
            }
            0xFF55 if !self.running => {
                self.remaining = (val & 0x7F) as u16 + 1;
                self.offset = 0;
                self.block_active = false;
                self.hblank = val & 0x80 != 0;
                self.running = self.dst < 0x2000;
                self.cancelled = false;
            }
            _ => (),
        }
    }

    pub fn begin_hblank(&mut self) {
        if self.running && self.hblank { self.block_active = true; }
    }

    pub fn is_hblank(&self) -> bool { self.hblank }

    pub fn active(&self) -> bool { self.running }

    pub fn next_pair(&self) -> Option<(u16, u16)> {
        (self.running && (!self.hblank || self.block_active)).then(|| (self.src.wrapping_add(self.offset), self.dst + self.offset))
    }

    pub fn advance_pair(&mut self) {
        if !self.running { return; }
        self.offset += 2;
        if self.offset == 16 {
            self.block_active = false;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hblank_runs_one_block_and_cancel_preserves_remaining_length() {
        let mut dma = VramDma::default();
        dma.write(0xFF55, 0x82);
        assert!(dma.next_pair().is_none());
        dma.begin_hblank();
        for _ in 0..8 {
            assert!(dma.next_pair().is_some());
            dma.advance_pair();
        }
        assert!(dma.next_pair().is_none());
        assert_eq!(dma.read(), 1);
        dma.write(0xFF55, 0);
        assert!(!dma.active());
        assert_eq!(dma.read(), 0x81);
        dma.write(0xFF55, 0x80);
        assert_eq!(dma.read(), 0);
        dma.begin_hblank();
        for _ in 0..8 { dma.advance_pair(); }
        assert_eq!(dma.read(), 0xFF);
    }

    #[test]
    fn gdma_masks_addresses_and_transfers_in_eight_cycles_per_block() {
        let mut dma = VramDma::default();
        dma.write(0xFF51, 0xC1);
        dma.write(0xFF52, 0x2F);
        dma.write(0xFF53, 0xE1);
        dma.write(0xFF54, 0x3F);
        dma.write(0xFF55, 1);
        assert_eq!(dma.read(), 1);
        for i in 0..16 {
            let (src, dst) = dma.next_pair().unwrap();
            assert_eq!(src, 0xC120 + i * 2);
            assert_eq!(dst, 0x130 + i * 2);
            dma.advance_pair();
            assert_eq!(dma.active(), i != 15);
        }
        assert_eq!(dma.read(), 0xFF);
        assert_eq!(dma.src, 0xC140);
        assert_eq!(dma.dst, 0x150);
    }

    #[test]
    fn transfer_stops_at_vram_end_without_wrapping() {
        let mut dma = VramDma::default();
        dma.write(0xFF53, 0x1F);
        dma.write(0xFF54, 0xF0);
        dma.write(0xFF55, 0x7F);
        for _ in 0..8 { dma.advance_pair(); }
        assert!(!dma.active());
        assert!(dma.next_pair().is_none());
    }
}
