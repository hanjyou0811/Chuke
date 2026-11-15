use crate::{cpu::interrupts::{self, Interrupts}, ppu::Mode::Drawing};

// lcdcレジスタ
const PPU_ENABLE: u8 = 1 << 7;
const WINDOW_TILE_MAP: u8 = 1 << 6;
const WINDOW_ENABLE: u8 = 1 << 5;
const TILE_DATA_ADDRESSING_MODE: u8 = 1 << 4;
const BG_TILE_MAP: u8 = 1 << 3;
const SPRITE_SIZE: u8 = 1 << 2;
const SPRITE_ENABLE: u8 = 1 << 1;
const BG_WINDOW_ENABLE: u8 = 1 << 0;

// statレジスタ
const LYC_EQ_LY_INT: u8 = 1 << 6;
const OAM_SCAN_INT: u8 = 1 << 5;
const VBLANK_INT: u8 = 1 << 4;
const HBLANK_INT: u8 = 1 << 3;
const LYC_EQ_LY: u8 = 1 << 2;

// sprite flags
const OBJ2BG_PRIORIY: u8 = 1 << 7;
const Y_FLIP: u8 = 1 << 6;
const X_FLIP: u8 = 1 << 5;
const PALETTE: u8 = 1 << 4;

pub const LCD_WIDTH: usize = 160;
pub const LCD_HEIGHT: usize = 144;

#[derive(Copy, Clone, PartialEq, Eq)]
enum Mode {
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Drawing = 3,
}
pub struct Ppu {
    mode: Mode,
    // ff40 lcdcレジスタ
    lcdc: u8,
    // ff41 statレジスタ 割り込みとか
    stat: u8,
    // ff42 scy
    // ff43 scx
    scy: u8,
    scx: u8,
    // ff44 lyレジスタ 今何行目のレンダリングをしているか
    ly: u8,
    // ff45 lycレジスタ ly == lycで割り込み
    lyc: u8,
    // ff47, bg/window
    // ff48, obp0 パレットデータ
    // ff49, obp1 パレットデータ
    bgp: u8,
    obp0: u8,
    obp1: u8,

    // ff4a wyレジスタ
    // ff4b wxレジスタ
    wy: u8,
    wx: u8,

    // 0x8000~0x9FFF VRAM
    // 0xFE00~0xFE9F OAM
    vram: Box<[[u8; 0x2000]; 2]>,
    vram_bank: usize,
    oam: Box<[u8; 0xA0]>,
    buffer: Box<[[u8; 3]; LCD_WIDTH * LCD_HEIGHT]>,

    cycles: usize,

    wly: u8,

    pub oam_dma: Option<u16>,

    // for CGB
    is_cgb: bool,
    cgb_hardware: bool, 
    // ff68 bg palette index
    // ff69 bg palette data
    bgpi: u8,
    bgpd: Box<[u8; 0x40]>,

    // ff6a obj palette index
    // ff6b obj palette data
    obpi: u8, 
    obpd: Box<[u8; 0x40]>,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Sprite {
    y: u8,
    x: u8,
    tile_idx: u8,
    flags: u8,
}

impl Ppu {
    pub fn new(is_cgb: bool) -> Self {
        Self {
            mode: Mode::OamScan,
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            wy: 0,
            wx: 0,
            vram: Box::new([[0; 0x2000]; 2]),
            vram_bank: 0,
            oam: Box::new([0; 0xA0]),
            buffer: Box::new([[0; 3]; LCD_WIDTH * LCD_HEIGHT]),
            cycles: 20,
            wly: 0,
            oam_dma: None,

            is_cgb,
            cgb_hardware: is_cgb,
            bgpi: 0,
            bgpd: Box::new([0; 0x40]),
            obpi: 0, 
            obpd: Box::new([0; 0x40]),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            // vram
            0x8000..=0x9FFF => if self.mode == Mode::Drawing {
                0xFF
            } else {
                self.vram[self.vram_bank][addr as usize & 0x1FFF]
            },
            // oam
            0xFE00..=0xFE9F => if self.mode == Mode::Drawing || self.mode == Mode::OamScan {
                0xFF
            } else {
                if self.oam_dma.is_some() {
                    0xFF
                }else {
                    self.oam[addr as usize & 0xFF]
                }
            },
            0xFF40 => self.lcdc,
            0xFF41 => 0x80 | self.stat | self.mode as u8,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            0xFF4F => 0xFE | self.vram_bank as u8,
            0xFF68 => self.bgpi,
            0xFF69 => if self.mode != Mode::Drawing {
                self.bgpd[(self.bgpi & 0x3F) as usize]
            } else {
                0xFF
            },
            0xFF6A => self.obpi,
            0xFF6B => if self.mode != Mode::Drawing {
                self.obpd[(self.obpi & 0x3F) as usize]
            } else {
                0xFF
            },
            _ => unreachable!(),
        }
    }
    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            // vram
            0x8000..=0x9FFF => if self.mode == Mode::Drawing {
                // アクセスできない
            } else {
                self.vram[self.vram_bank][addr as usize & 0x1FFF] = val;
            },
            // oam
            0xFE00..=0xFE9F => if self.mode == Mode::Drawing || self.mode == Mode::OamScan {
                // アクセスできない
            } else {
                if self.oam_dma.is_none() {
                    self.oam[addr as usize & 0xFF] = val;
                }
            },
            0xFF40 => self.lcdc = val,
            0xFF41 => self.stat = (self.stat & LYC_EQ_LY) | (val & 0xF8),
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF45 => self.lyc = val,
            0xFF46 => self.oam_dma = Some((val as u16) << 8),
            0xFF47 => self.bgp = val,
            0xFF48 => self.obp0 = val,
            0xFF49 => self.obp1 = val,
            0xFF4A => self.wy = val,
            0xFF4B => self.wx = val,
            0xFF4F => self.vram_bank = (val & 0x01) as usize,
            0xFF68 => self.bgpi = val & 0xBF,
            0xFF69 => if self.mode != Drawing {
                self.bgpd[(self.bgpi & 0x3F) as usize] = val;
                if self.bgpi & 0x80 != 0 {
                    self.bgpi = (self.bgpi & 0x80) | ((self.bgpi.wrapping_add(1))) & 0x3F;
                }
            },
            0xFF6A => self.obpi = val & 0xBF,
            0xFF6B => if self.mode != Drawing {
                self.obpd[(self.obpi & 0x3F) as usize] = val;
                if self.obpi & 0x80 != 0 {
                    self.obpi = (self.obpi & 0x80) | ((self.obpi.wrapping_add(1)) & 0x3F);
                }
            },
            _ => unreachable!("{}", addr),
        }
    }

    fn get_pixel_from_tile(&self, bank: usize, tile_idx: usize, row: u8, col: u8) -> u8 {
        let r = (row * 2) as usize;
        let c = (7 - col) as usize;
        let tile_addr = tile_idx << 4;
        let low = self.vram[bank][(tile_addr | r) & 0x1FFF];
        let high = self.vram[bank][(tile_addr | (r + 1)) & 0x1FFF];
        (((high >> c) & 1) << 1) | ((low >> c) & 1)
    }

    fn get_tile_idx_from_tile_map(&self, tile_map: bool, row: u8, col: u8) -> usize {
        let start_addr: usize = 0x1800 | ((tile_map as usize) << 10);
        let ret = self.vram[0][start_addr | (((row as usize) << 5) + col as usize)];
        if self.lcdc & TILE_DATA_ADDRESSING_MODE > 0 {
            ret as usize
        } else {
            ((ret as i8 as i16) + 0x100) as usize
        }
    }

    fn palette_color(memory: &[u8], palette: u8, color: u8) -> [u8; 3] {
        let index = palette as usize * 8 + color as usize * 2;
        let rgb = u16::from_le_bytes([memory[index], memory[index + 1]]);
        [0, 5, 10].map(|shift| {
            let component = ((rgb >> shift) & 31) as u8;
            (component << 3) | (component >> 2)
        })
    }

    fn monochrome_color(palette: u8, color: u8) -> [u8; 3] {
        [0xFF - ((palette >> (color * 2)) & 3) * 0x55; 3]
    }

    fn background_pixel(&self, map: bool, x: u8, y: u8) -> (u8, u8) {
        let map_addr = 0x1800 | ((map as usize) << 10)
            | ((y as usize >> 3) * 32 + (x as usize >> 3));
        let attr = if self.is_cgb { self.vram[1][map_addr] } else { 0 };
        let tile = self.get_tile_idx_from_tile_map(map, y >> 3, x >> 3);
        let row = (y & 7) ^ if attr & Y_FLIP != 0 { 7 } else { 0 };
        let col = (x & 7) ^ if attr & X_FLIP != 0 { 7 } else { 0 };
        let pixel = self.get_pixel_from_tile(((attr >> 3) & 1) as usize, tile, row, col);
        (pixel, attr)
    }

    fn background_color(&self, pixel: u8, attr: u8) -> [u8; 3] {
        if self.is_cgb {
            Self::palette_color(self.bgpd.as_ref(), attr & 7, pixel)
        } else if self.cgb_hardware {
            Self::palette_color(self.bgpd.as_ref(), 0, (self.bgp >> (pixel * 2)) & 3)
        } else {
            Self::monochrome_color(self.bgp, pixel)
        }
    }

    fn render_bg(&mut self, bg_prio: &mut [bool; LCD_WIDTH]) {
        if !self.is_cgb && self.lcdc & BG_WINDOW_ENABLE == 0 {
            for i in 0..LCD_WIDTH {
                self.buffer[self.ly as usize * LCD_WIDTH + i] = [255; 3];
                bg_prio[i] = false;
            }
            return;
        }
        let y = self.ly.wrapping_add(self.scy);
        for i in 0..LCD_WIDTH {
            let x = (i as u8).wrapping_add(self.scx);
            let (pixel, attr) = self.background_pixel(self.lcdc & BG_TILE_MAP != 0, x, y);
            self.buffer[self.ly as usize * LCD_WIDTH + i] = self.background_color(pixel, attr);
            bg_prio[i] = pixel != 0;
        }
    }

    fn render_window(&mut self, bg_prio: &mut [bool; LCD_WIDTH]) {
        if (!self.is_cgb && self.lcdc & BG_WINDOW_ENABLE == 0)
            || self.lcdc & WINDOW_ENABLE == 0 || self.wy > self.ly {
            return;
        }
        let mut visible = false;
        for i in 0..LCD_WIDTH {
            let x = i as i16 + 7 - self.wx as i16;
            if x < 0 { continue; }
            visible = true;
            let (pixel, attr) = self.background_pixel(self.lcdc & WINDOW_TILE_MAP != 0, x as u8, self.wly);
            self.buffer[self.ly as usize * LCD_WIDTH + i] = self.background_color(pixel, attr);
            bg_prio[i] = pixel != 0;
        }
        self.wly += visible as u8;
    }

    fn render_sprite(&mut self, bg_prio: &[bool; LCD_WIDTH]) {
        if self.lcdc & SPRITE_ENABLE == 0 {
            return ;
        }
        let size = if self.lcdc & SPRITE_SIZE > 0 { 16 } else { 8 };

        let mut sprites: Vec<Sprite> = unsafe {
            std::mem::transmute::<[u8; 0xA0], [Sprite; 40]>(
                self.oam.as_ref().clone()
            )
        }.into_iter().filter_map(|mut sprite| {
            sprite.y = sprite.y.wrapping_sub(16);
            sprite.x = sprite.x.wrapping_sub(8);
            if self.ly.wrapping_sub(sprite.y) < size {
                Some(sprite)
            } else {
                None
            }
        }).take(10).collect();
        sprites.reverse();
        sprites.sort_by(|&a, &b| b.x.cmp(&a.x));

        for sprite in sprites {
            let palette = if sprite.flags & PALETTE > 0 { self.obp1 } else { self.obp0 };
            let mut tile_idx = sprite.tile_idx as usize;
            let mut row = if sprite.flags & Y_FLIP > 0 {
                size - 1 - self.ly.wrapping_sub(sprite.y)
            } else {
                self.ly.wrapping_sub(sprite.y)
            };
            if size == 16 {
                tile_idx &= 0xFE;
            } 
            tile_idx += (row >= 8) as usize;
            row &= 7;

            for col in 0..8 {
                let col_flipped = if sprite.flags & X_FLIP > 0 {
                    7 - col
                } else {
                    col
                };
                let pixel = self.get_pixel_from_tile(0, tile_idx, row, col_flipped);
                let i = sprite.x.wrapping_add(col) as usize;
                if i < LCD_WIDTH && pixel > 0 {
                    if sprite.flags & OBJ2BG_PRIORIY == 0 || bg_prio[i] == false {
                        self.buffer[(self.ly as usize * LCD_WIDTH) + i] = Self::monochrome_color(palette, pixel);
                    }
                }
            }
        }
    }

    pub fn oam_dma_emulate_cycle(&mut self, val: u8) {
        if let Some(addr) = self.oam_dma {
            if self.mode == Mode::Drawing || self.mode == Mode::OamScan {

            } else {
                self.oam[addr as usize & 0xFF] = val;
            }
            self.oam_dma = Some(addr.wrapping_add(1)).filter(| &x| (x as u8) < 0xA0);
        }
    }

    fn check_lyc_eq_ly(&mut self, interrupts: &mut Interrupts) {
        if self.ly == self.lyc {
            self.stat |= LYC_EQ_LY;
            if self.stat & LYC_EQ_LY_INT > 0 {
                interrupts.irq(interrupts::STAT);
            }
        } else {
            self.stat &= !LYC_EQ_LY;
        }
    }

    pub fn emulate_cycle(&mut self, interrupts: &mut Interrupts) -> bool {
        if self.lcdc & PPU_ENABLE == 0 {
            return false;
        }

        self.cycles -= 1;
        if self.cycles > 0 {
            return false;
        }
        
        let mut ret = false;
        let mut bg_prio = [false; LCD_WIDTH];
        match self.mode {
            Mode::HBlank => {
                self.ly += 1;
                if self.ly < 144 {
                    self.mode = Mode::OamScan;
                    self.cycles = 20;
                    if self.stat & OAM_SCAN_INT > 0 {
                        interrupts.irq(interrupts::STAT);
                    }
                } else {
                    self.mode = Mode::VBlank;
                    self.cycles = 114;
                    interrupts.irq(interrupts::VBLANK);
                    if self.stat & VBLANK_INT > 0 {
                        interrupts.irq(interrupts::STAT);
                    }
                }
                self.check_lyc_eq_ly(interrupts);
            },
            Mode::VBlank => {
                self.ly += 1;
                if self.ly > 153 {
                    ret = true;
                    self.ly = 0;
                    self.wly = 0;
                    self.mode = Mode::OamScan;
                    self.cycles = 20;
                    if self.stat & OAM_SCAN_INT > 0 {
                        interrupts.irq(interrupts::STAT);
                    }
                } else {
                    self.cycles = 114;
                }
                self.check_lyc_eq_ly(interrupts);
            },
            Mode::OamScan => {
                self.mode = Mode::Drawing;
                self.cycles = 43;
            },
            Mode::Drawing => {
                self.render_bg(&mut bg_prio);
                self.render_window(&mut bg_prio);
                self.render_sprite(&mut bg_prio);
                self.mode = Mode::HBlank;
                self.cycles = 51;
                if self.stat & HBLANK_INT > 0{
                    interrupts.irq(interrupts::STAT);
                }
            },
        }
        ret
    }

    pub fn pixel_buffer(&self) -> Box<[u8]> {
        self.buffer.iter().flatten().copied().collect()
    }

    pub fn set_mode(&mut self, is_cgb: bool) {
        self.is_cgb = is_cgb;
    }
}
