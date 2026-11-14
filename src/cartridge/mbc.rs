pub enum Mbc {
    Mbc5 {
        sram_enable: bool,
        rom_bank: usize,
        ram_bank: usize,
        rumble: bool,
    },
    NoMbc,
    Mbc1 {
        sram_enable: bool,
        low_bank: usize,
        high_bank: usize,
        bank_mode: bool,
        rom_banks: usize,
    },
    Mbc2 {
        sram_enable: bool,
        low_bank: usize,
        high_bank: usize,
        bank_mode: bool,
        rom_banks: usize,
    },
    Mbc3 {
        sram_enable: bool,
        low_bank: usize,
        ram_bank: usize,
        rom_banks: usize,
        // for rtc
        rtc_mode: bool,
    },
}
impl Mbc {
    pub fn new(cartridge_type: u8, rom_banks: usize) -> Self {
        match cartridge_type {
            0x00 | 0x08 | 0x09 => Self::NoMbc,
            0x01..=0x03        => Self::Mbc1 {
                sram_enable: false,
                low_bank: 0b0001,
                high_bank: 0b00,
                bank_mode: false,
                rom_banks,
            },
            0x05..=0x06 => Self::Mbc2 {
                sram_enable: false,
                low_bank: 0b0001,
                high_bank: 0b00,
                bank_mode: false,
                rom_banks: 16,
            },
            0x0F..=0x13 => Self::Mbc3 {
                sram_enable: false,
                low_bank: 0b0001,
                ram_bank: 0b00,
                rom_banks,
                rtc_mode: false,
            },
            0x19..=0x1E => Self::Mbc5 {
                sram_enable: false,
                rom_bank: 1,
                ram_bank: 0,
                rumble: cartridge_type >= 0x1C,
            },
            _                   => panic!("Not Supported: {:02x}", cartridge_type),
        }
    }
    pub fn write(&mut self, addr: u16, val: u8) {
        match *self {
            Self::Mbc5 { ref mut sram_enable, ref mut rom_bank, ref mut ram_bank, rumble } => match addr {
                0x0000..=0x1FFF => *sram_enable = val & 0x0F == 0x0A,
                0x2000..=0x2FFF => *rom_bank = (*rom_bank & 0x100) | val as usize,
                0x3000..=0x3FFF => *rom_bank = (*rom_bank & 0xFF) | ((val as usize & 1) << 8),
                0x4000..=0x5FFF => *ram_bank = val as usize & if rumble { 7 } else { 15 },
                0x6000..=0x7FFF => (),
                _ => unreachable!(),
            },
            Self::NoMbc => {},
            Self::Mbc1 {
                ref mut sram_enable,
                ref mut low_bank,
                ref mut high_bank,
                ref mut bank_mode,
                ..
            } => match addr {
                0x0000..=0x1FFF => *sram_enable = val & 0xF == 0xA,
                0x2000..=0x3FFF => *low_bank = if val & 0b11111 == 0b00000 {
                    0b00001
                } else {
                    (val & 0b11111) as usize
                },
                0x4000..=0x5FFF => *high_bank = (val & 0b11) as usize,
                0x6000..=0x7FFF => *bank_mode = val & 0b1 > 0,
                _               => unreachable!(),
            },
            Self::Mbc2 {
                ref mut sram_enable,
                ref mut low_bank,
                ref mut high_bank,
                ref mut bank_mode,
                ..
            } => match addr {
                0x0000..=0x1FFF => *sram_enable = val & 0xF == 0xA,
                0x2000..=0x3FFF => *low_bank = if val & 0b11111 == 0b00000 {
                    0b00001
                } else {
                    (val & 0b11111) as usize
                },
                0x4000..=0x5FFF => *high_bank = (val & 0b11) as usize,
                0x6000..=0x7FFF => *bank_mode = val & 0b1 > 0,
                _               => unreachable!(),
            },
            Self::Mbc3 {
                ref mut sram_enable,
                ref mut low_bank,
                ref mut ram_bank,
                ref mut rtc_mode,
                ..
            } => match addr {
                0x0000..=0x1FFF => *sram_enable = val & 0xF == 0xA,
                0x2000..=0x3FFF => *low_bank = if val & 0x7f == 0 {
                    0b00001
                } else {
                    (val & 0x7f) as usize
                },
                0x4000..=0x5FFF => {
                    match val {
                        0x00..=0x07 => {
                            *ram_bank = (val & 0b11) as usize;
                            *rtc_mode = false;
                        },0x08..=0x0C => {
                            *rtc_mode = true;
                        },
                        _ => {
                            unreachable!("{:x}", val);
                        }
                    }
                },
                0x6000..=0x7FFF => {},
                _ => unreachable!()
            },
        }
    }
    pub fn get_addr(&self, addr: u16) -> usize {
        match self {
            Self::Mbc5 { rom_bank, ram_bank, .. } => match addr {
                0x0000..=0x3FFF => addr as usize,
                0x4000..=0x7FFF => (rom_bank << 14) | (addr as usize & 0x3FFF),
                0xA000..=0xBFFF => (ram_bank << 13) | (addr as usize & 0x1FFF),
                _ => unreachable!(),
            },
            Self::NoMbc => addr as usize,
            Self::Mbc1 {
                low_bank,
                high_bank,
                bank_mode,
                rom_banks,
                ..
            } => {
                match addr {
                    0x0000..=0x3FFF => {
                        if *bank_mode {
                            (*high_bank << 19) | (addr & 0x3FFF) as usize
                        } else {
                            (addr & 0x3FFF) as usize
                        }
                    },
                    0x4000..=0x7FFF => {
                        (*high_bank << 19) | ((*low_bank & (*rom_banks - 1) ) << 14) | (addr & 0x3FFF) as usize
                    },
                    0xA000..=0xBFFF => {
                        if *bank_mode {
                            (*high_bank << 13) | (addr & 0x1FFF) as usize
                        } else {
                            (addr & 0x1FFF) as usize
                        }
                    },
                    _ => unreachable!(),
                }
            },
            Self::Mbc2 {
                low_bank,
                high_bank,
                bank_mode,
                rom_banks,
                ..
            } => {
                match addr {
                    0x0000..=0x3FFF => {
                        (addr & 0x3FFF) as usize
                    },
                    0x4000..=0x7FFF => {
                        (*high_bank << 19) | ((*low_bank & (*rom_banks - 1) ) << 14) | (addr & 0x3FFF) as usize
                    },
                    0xA000..=0xBFFF => {
                        if *bank_mode {
                            (*high_bank << 13) | (addr & 0x1FFF) as usize
                        } else {
                            (addr & 0x1FFF) as usize
                        }
                    },
                    _ => unreachable!(),
                }
            },
            Self::Mbc3 {
                low_bank,
                ram_bank,
                rom_banks,
                ..
            } => {
                match addr {
                    0x0000..=0x3FFF => {
                        (addr & 0x3FFF) as usize
                    },
                    0x4000..=0x7FFF => {
                        ((*low_bank & (*rom_banks - 1) )<< 14) | (addr & 0x3FFF) as usize
                    },
                    0xA000..=0xBFFF => {
                        (*ram_bank << 13) | (addr & 0x1FFF) as usize
                    },
                    _ => unreachable!(),
                }
            }
        }
    }
}

#[cfg(test)]
mod mbc5_tests {
    use super::*;

    #[test]
    fn all_nine_rom_bank_bits_and_bank_zero_are_selectable() {
        let mut mbc = Mbc::new(0x19, 512);
        for bank in [0, 1, 255, 256, 511] {
            mbc.write(0x2000, bank as u8);
            mbc.write(0x3000, (bank >> 8) as u8);
            assert_eq!(mbc.get_addr(0x4000), bank * 0x4000);
            assert_eq!(mbc.get_addr(0), 0);
        }
    }

    #[test]
    fn rumble_bit_is_not_a_ram_bank_bit() {
        let mut plain = Mbc::new(0x1B, 2);
        let mut rumble = Mbc::new(0x1E, 2);
        plain.write(0x4000, 15);
        rumble.write(0x4000, 15);
        assert_eq!(plain.get_addr(0xA000), 15 * 0x2000);
        assert_eq!(rumble.get_addr(0xA000), 7 * 0x2000);
    }
}
