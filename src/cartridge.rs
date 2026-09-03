mod mbc;
use mbc::Mbc;
use crate::save::SaveManager;
use std::{io, path::PathBuf};

#[repr(C)]
pub struct CartridgeHeader {
    entry_point: [u8; 4],
    logo: [u8; 48],
    title: [u8; 11],
    maker: [u8; 4],
    pub cgb_flag: [u8; 1],
    new_licensee: [u8; 2],
    sgb_flag: [u8; 1],
    cartridge_type: [u8; 1],
    rom_size: [u8; 1],
    sram_size: [u8; 1],
    destination: [u8; 1],
    old_licensee: [u8; 1],
    game_version: [u8; 1],
    header_checksum: [u8; 1],
    global_checksum: [u8; 2],
}
impl CartridgeHeader {
    fn new(data: [u8; 0x50]) -> Self {
        let ret = unsafe {
            std::mem::transmute::<[u8; 0x50], Self>(data)
        };
        let mut chksum: u8 = 0;
        for i in 0x34..=0x4c {
            chksum = chksum.wrapping_sub(data[i]).wrapping_sub(1);
        }
        assert!(chksum == ret.header_checksum[0], "Chksum validation failed.");
        ret
    }

    fn rom_size(&self) -> usize {
        assert!(self.rom_size[0] <= 0x08, "Invalid rom size {}", self.rom_size[0]);
        return 1 << (15 + self.rom_size[0]);
    }

    fn sram_size(&self) -> usize {
        match self.sram_size[0] {
            0x00 => 0,
            0x01 => 0x800,
            0x02 => 0x2000,
            0x03 => 0x8000,
            0x04 => 0x20000,
            0x05 => 0x10000,
            _    => panic!("Invalid sram size {}", self.sram_size[0]),
        }
    }
}

pub struct Cartridge {
    rom: Box<[u8]>,
    sram: Box<[u8]>,
    mbc: Mbc,
    is_cgb: bool, 
    save: SaveManager,
    pub header: CartridgeHeader,
}
impl Cartridge {
    pub fn new(rom: Box<[u8]>, save_path: PathBuf) -> Self {
        let header = CartridgeHeader::new(rom[0x100..0x150].try_into().unwrap());
        let title = str::from_utf8(&header.title).unwrap().trim_end_matches('\0').to_string();
        let rom_size = header.rom_size();
        let rom_banks = rom_size >> 14;
        let mbc = Mbc::new(header.cartridge_type[0], rom_banks);
        let sram_size = match mbc {
            Mbc::Mbc2 { .. } => 512,
            _ => header.sram_size(),
        };
        let has_battery = matches!(header.cartridge_type[0], 0x03 | 0x06 | 0x09 | 0x0F | 0x10 | 0x13);

        println!(
            "Cartridge info {{ title: {}, type: {}, is_cgb: {}, rom_size: {} B, sram_size: {} B, }}", 
            title,
            match mbc {
                Mbc::NoMbc { .. } => "No Mbc",
                Mbc::Mbc1 { .. }  => "Mbc1",
                Mbc::Mbc2 { .. } => "Mbc2",
                Mbc::Mbc3 { .. } => "Mbc3",
            },
            matches!(header.cgb_flag[0], 0x80 | 0xC0),
            rom_size,
            sram_size,
        );

        assert!(rom.len() == rom_size, "Expected {} bytes of cartridge ROM, got {}", 
            rom_size, 
            rom.len()
        );

        let mut cartridge = Self {
            rom,
            sram: vec![0; sram_size].into(),
            mbc,
            is_cgb: matches!(header.cgb_flag[0], 0x80 | 0xC0),
            // is_cgb: true,
            save: SaveManager::new(save_path, has_battery),
            header,
        };
        if let Err(error) = cartridge.save.load(&mut cartridge.sram) {
            eprintln!("Failed to load save data: {error}");
        }
        cartridge
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.rom[self.mbc.get_addr(addr) & (self.rom.len() - 1)],
            0xA000..=0xBFFF => match self.mbc {
                Mbc::NoMbc if !self.sram.is_empty() => self.sram[addr as usize & (self.sram.len() - 1)],
                Mbc::NoMbc => 0xFF,
                Mbc::Mbc1  { ref sram_enable, .. } => if *sram_enable && !self.sram.is_empty() {
                    self.sram[self.mbc.get_addr(addr) & (self.sram.len() - 1)]
                } else {
                    0xFF
                },
                Mbc::Mbc2  { ref sram_enable, .. } => if *sram_enable {
                    self.sram[self.mbc.get_addr(addr) & (self.sram.len() - 1)]
                } else {
                    0xFF
                },
                Mbc::Mbc3  {
                    ref sram_enable, 
                    ref rtc_mode,    
                    .. 
                } => if *sram_enable && !*rtc_mode && !self.sram.is_empty() {
                    self.sram[self.mbc.get_addr(addr) & (self.sram.len() - 1)]
                } else {
                    0xFF
                },
            },
            _               => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x7FFF => self.mbc.write(addr, val),
            0xA000..=0xBFFF => {
                let index = match self.mbc {
                Mbc::NoMbc if !self.sram.is_empty() => Some(addr as usize & (self.sram.len() - 1)),
                Mbc::NoMbc => None,
                Mbc::Mbc1  { ref sram_enable, .. } => if *sram_enable && !self.sram.is_empty() {
                    Some(self.mbc.get_addr(addr) & (self.sram.len() - 1))
                } else { None },
                Mbc::Mbc2  { ref sram_enable, .. } => if *sram_enable {
                    Some(self.mbc.get_addr(addr) & (self.sram.len() - 1))
                } else { None },
                Mbc::Mbc3  { ref sram_enable, ref rtc_mode, .. } => if *sram_enable && !*rtc_mode && !self.sram.is_empty() {
                    Some(self.mbc.get_addr(addr) & (self.sram.len() - 1))
                } else { None },
                };

                if let Some(index) = index && self.sram[index] != val {
                    self.sram[index] = val;
                    self.save.mark_dirty();
                }
            },
            _               => unreachable!(),
        }
    }

    pub fn is_cgb(&self) -> bool {
        self.is_cgb
    }


    pub fn save_if_ready(&mut self, current_frame: u64) -> io::Result<()> {
        if self.save.save_ready(current_frame) {
            self.save.save(&self.sram, current_frame)?;
        }
        Ok(())
    }

    pub fn flush_save(&mut self, current_frame: u64) -> io::Result<()> {
        self.save.save(&self.sram, current_frame)
    }
}