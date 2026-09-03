#[derive(Clone, Copy, Debug, Default)]
pub struct Registers {
    pub pc: u16,
    pub sp: u16,
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,
}
impl Registers {
    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16)
    }
    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }
    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }
    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    pub fn write_af(&mut self, val: u16) {
        self.a = val.to_be_bytes()[0];
        self.f = val.to_be_bytes()[1] & 0xF0;
    }
    pub fn write_bc(&mut self, val: u16) {
        self.b = val.to_be_bytes()[0];
        self.c = val.to_be_bytes()[1];
    }
    pub fn write_de(&mut self, val: u16) {
        self.d = val.to_be_bytes()[0];
        self.e = val.to_be_bytes()[1];
    }
    pub fn write_hl(&mut self, val: u16) {
        self.h = val.to_be_bytes()[0];
        self.l = val.to_be_bytes()[1];
    }

    pub fn zf(&mut self) -> bool {
        (self.f & 0b_1000_0000) > 0
    }

    pub fn nf(&mut self) -> bool {
        (self.f & 0b_0100_0000) > 0
    }
    pub fn hf(&mut self) -> bool {
        (self.f & 0b_0010_0000) > 0
    }
    pub fn cf(&mut self) -> bool {
        (self.f & 0b_0001_0000) > 0
    }

    pub fn set_zf(&mut self, zf: bool) {
        if zf {
            self.f |= 0b_1000_0000;
        } else {
            self.f &= 0b_0111_1111;
        }
    }
    pub fn set_nf(&mut self, nf: bool) {
        if nf {
            self.f |= 0b_0100_0000;
        } else {
            self.f &= 0b_1011_1111;
        }
    }
    pub fn set_hf(&mut self, hf: bool) {
        if hf {
            self.f |= 0b_0010_0000;
        } else {
            self.f &= 0b_1101_1111;
        }
    }
    pub fn set_cf(&mut self, cf: bool) {
        if cf {
            self.f |= 0b_0001_0000;
        } else {
            self.f &= 0b_1110_1111;
        }
    }
}
