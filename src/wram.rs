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
#[cfg(test)]
mod tests {
    use super::WRam;

    #[test]
    fn banks_and_echo_are_independent_and_zero_selects_one() {
        let mut ram = WRam::new();
        ram.write(0xC000, 0x11);
        for bank in 1..=7 {
            ram.write(0xFF70, bank);
            ram.write(0xD123, bank);
            assert_eq!(ram.read(0xF123), bank);
        }
        for bank in 1..=7 {
            ram.write(0xFF70, bank);
            assert_eq!(ram.read(0xD123), bank);
            assert_eq!(ram.read(0xE000), 0x11);
        }
        ram.write(0xFF70, 0xF8);
        assert_eq!(ram.read(0xFF70), 0xF8);
        assert_eq!(ram.read(0xD123), 1);
        ram.write(0xF123, 0x22);
        ram.write(0xFF70, 1);
        assert_eq!(ram.read(0xD123), 0x22);
    }
}
