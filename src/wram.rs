use std::cmp::max;

pub struct WRam {
    svbk: u8,
    ram: Box<[[u8; 0x1000];8]>,
}
impl WRam {
    pub fn new() -> Self {
        Self {
            svbk: 0,
            ram: Box::new([[0; 0x1000]; 8]),
        }
    }
    pub fn read(&self, addr: u16) -> u8 {
        if addr == 0xFF70 {
            return self.svbk | 0b1111_1000;
        }
        match addr {
            0xC000..=0xCFFF | 0xE000..=0xEFFF => {
                self.ram[0][(addr & 0x0FFF) as usize]
            },
            0xD000..=0xDFFF | 0xF000..=0xFDFF => {
                self.ram[max(1, self.svbk) as usize][(addr & 0x0FFF) as usize]
            },
            _ => unreachable!()
        }
    }
    pub fn write(&mut self, addr: u16, val: u8) {
        if addr == 0xFF70 {
            self.svbk = val & 0b111;
            return ;
        }
        match addr {
            0xC000..=0xCFFF | 0xE000..=0xEFFF => {
                self.ram[0][(addr & 0x0FFF) as usize] = val;
            },
            0xD000..=0xDFFF | 0xF000..=0xFDFF => {
                self.ram[max(1, self.svbk) as usize][(addr & 0x0FFF) as usize] = val;
            },
            _ => unreachable!()
        }
    }
}
