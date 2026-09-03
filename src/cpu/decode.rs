use crate::cpu::Cpu;
use crate::bus::Bus;
use crate::cpu::operand::{Cond, Direct8, Direct16, IO8, Imm8, Imm16, Indirect, Reg8, Reg16};

impl Cpu {
    pub fn decode(&mut self, bus: &mut Bus) {
        if self.ctx.cb {
            self.cb_decode(bus);
            return ;
        }
        match self.ctx.opcode {
            0x00 => self.nop(bus),
            0x01 => self.ld16(bus, Reg16::BC, Imm16),
            0x02 => self.ld(bus, Indirect::BC, Reg8::A),
            0x03 => self.inc16(bus, Reg16::BC),
            0x04 => self.inc(bus, Reg8::B),
            0x05 => self.dec(bus, Reg8::B),
            0x06 => self.ld(bus,Reg8::B, Imm8),
            0x07 => self.rlca(bus),
            0x08 => self.ld16(bus, Direct16, Reg16::SP),
            0x09 => self.add_hl_reg16(bus, Reg16::BC),
            0x0A => self.ld(bus, Reg8::A, Indirect::BC),
            0x0B => self.dec16(bus, Reg16::BC),
            0x0C => self.inc(bus, Reg8::C),
            0x0D => self.dec(bus, Reg8::C),
            0x0E => self.ld(bus, Reg8::C, Imm8),
            0x0F => self.rrca(bus),

            0x10 => self.stop(bus),
            0x11 => self.ld16(bus, Reg16::DE, Imm16),
            0x12 => self.ld(bus, Indirect::DE, Reg8::A),
            0x13 => self.inc16(bus, Reg16::DE),
            0x14 => self.inc(bus, Reg8::D),
            0x15 => self.dec(bus, Reg8::D),
            0x16 => self.ld(bus, Reg8::D, Imm8),
            0x17 => self.rla(bus),
            0x18 => self.jr(bus),
            0x19 => self.add_hl_reg16(bus, Reg16::DE),
            0x1A => self.ld(bus,Reg8::A, Indirect::DE),
            0x1B => self.dec16(bus, Reg16::DE),
            0x1C => self.inc(bus, Reg8::E),
            0x1D => self.dec(bus, Reg8::E),
            0x1E => self.ld(bus, Reg8::E, Imm8),
            0x1F => self.rra(bus),

            0x20 => self.jr_c(bus, Cond::NZ),
            0x21 => self.ld16(bus,Reg16::HL, Imm16),
            0x22 => self.ld(bus,Indirect::HLI, Reg8::A),
            0x23 => self.inc16(bus, Reg16::HL),
            0x24 => self.inc(bus, Reg8::H),
            0x25 => self.dec(bus, Reg8::H),
            0x26 => self.ld(bus, Reg8::H, Imm8),
            0x27 => self.daa(bus),
            0x28 => self.jr_c(bus, Cond::Z),
            0x29 => self.add_hl_reg16(bus, Reg16::HL),
            0x2A => self.ld(bus, Reg8::A, Indirect::HLI),
            0x2B => self.dec16(bus,Reg16::HL),
            0x2C => self.inc(bus, Reg8::L),
            0x2D => self.dec(bus, Reg8::L),
            0x2E => self.ld(bus,Reg8::L, Imm8),
            0x2F => self.cpl(bus),

            0x30 => self.jr_c(bus, Cond::NC),
            0x31 => self.ld16(bus, Reg16::SP, Imm16),
            0x32 => self.ld(bus, Indirect::HLD, Reg8::A),
            0x33 => self.inc16(bus, Reg16::SP),
            0x34 => self.inc(bus, Indirect::HL),
            0x35 => self.dec(bus, Indirect::HL),
            0x36 => self.ld(bus, Indirect::HL, Imm8),
            0x37 => self.scf(bus),
            0x38 => self.jr_c(bus, Cond::C),
            0x39 => self.add_hl_reg16(bus, Reg16::SP),
            0x3A => self.ld(bus, Reg8::A, Indirect::HLD),
            0x3B => self.dec16(bus, Reg16::SP),
            0x3C => self.inc(bus, Reg8::A),
            0x3D => self.dec(bus, Reg8::A),
            0x3E => self.ld(bus, Reg8::A, Imm8),
            0x3F => self.ccf(bus),

            0x40..=0x45 | 0x47=> self.ld(bus, Reg8::B, self.decode_src()),
            0x46 => self.ld(bus, Reg8::B, Indirect::HL),
            0x48..=0x4D | 0x4F => self.ld(bus, Reg8::C, self.decode_src()),
            0x4E => self.ld(bus, Reg8::C, Indirect::HL),

            0x50..=0x55 | 0x57 => self.ld(bus, Reg8::D, self.decode_src()),
            0x56 => self.ld(bus, Reg8::D, Indirect::HL),
            0x58..=0x5D | 0x5F => self.ld(bus, Reg8::E, self.decode_src()),
            0x5E => self.ld(bus, Reg8::E, Indirect::HL),

            0x60..=0x65 | 0x67 => self.ld(bus, Reg8::H, self.decode_src()),
            0x66 => self.ld(bus, Reg8::H, Indirect::HL),
            0x68..=0x6D | 0x6F => self.ld(bus, Reg8::L, self.decode_src()),
            0x6E => self.ld(bus, Reg8::L, Indirect::HL),

            0x70..=0x75 | 0x77 => self.ld(bus, Indirect::HL, self.decode_src()),
            0x78..=0x7D | 0x7F => self.ld(bus, Reg8::A, self.decode_src()),
            0x76 => self.halt(bus),
            0x7E => self.ld(bus, Reg8::A, Indirect::HL),

            0x80..=0x85 | 0x87 => self.add(bus, self.decode_src()),
            0x86 => self.add(bus, Indirect::HL),
            0x88..=0x8D | 0x8F => self.adc(bus, self.decode_src()),
            0x8E => self.adc(bus, Indirect::HL),

            0x90..=0x95 | 0x97 => self.sub(bus, self.decode_src()),
            0x96 => self.sub(bus, Indirect::HL),
            0x98..=0x9D | 0x9F => self.sbc(bus, self.decode_src()),
            0x9E => self.sbc(bus, Indirect::HL),
        
            0xA0..=0xA5 | 0xA7 => self.and(bus, self.decode_src()),
            0xA6 => self.and(bus, Indirect::HL),
            0xA8..=0xAD | 0xAF => self.xor(bus, self.decode_src()),
            0xAE => self.xor(bus, Indirect::HL),

            0xB0..=0xB5 | 0xB7 => self.or(bus,self.decode_src()),
            0xB6 => self.or(bus, Indirect::HL),
            0xB8..=0xBD | 0xBF => self.cp(bus, self.decode_src()),
            0xBE => self.cp(bus, Indirect::HL),

            0xC0 => self.ret_c(bus, Cond::NZ),
            0xC1 => self.pop(bus, Reg16::BC),
            0xC2 => self.jp_c(bus, Cond::NZ),
            0xC3 => self.jp(bus),
            0xC4 => self.call_c(bus, Cond::NZ),
            0xC5 => self.push(bus, Reg16::BC),
            0xC6 => self.add(bus, Imm8),
            0xC7 => self.rst(bus, 0x00),
            0xC8 => self.ret_c(bus, Cond::Z),
            0xC9 => self.ret(bus),
            0xCA => self.jp_c(bus, Cond::Z),
            0xCB => self.cb_prefixed(bus),
            0xCC => self.call_c(bus, Cond::Z),
            0xCD => self.call(bus),
            0xCE => self.adc(bus, Imm8),
            0xCF => self.rst(bus, 0x08),

            0xD0 => self.ret_c(bus, Cond::NC),
            0xD1 => self.pop(bus, Reg16::DE),
            0xD2 => self.jp_c(bus, Cond::NC),
            0xD4 => self.call_c(bus, Cond::NC),
            0xD5 => self.push(bus, Reg16::DE),
            0xD6 => self.sub(bus, Imm8),
            0xD7 => self.rst(bus, 0x10),
            0xD8 => self.ret_c(bus, Cond::C),
            0xD9 => self.reti(bus),
            0xDA => self.jp_c(bus, Cond::C),
            0xDC => self.call_c(bus, Cond::C),
            0xDE => self.sbc(bus, Imm8),
            0xDF => self.rst(bus, 0x18),

            0xE0 => self.ld(bus, Direct8::DFF, Reg8::A),
            0xE1 => self.pop(bus, Reg16::HL),
            0xE2 => self.ld(bus, Indirect::CFF, Reg8::A),
            0xE5 => self.push(bus, Reg16::HL),
            0xE6 => self.and(bus, Imm8),
            0xE7 => self.rst(bus, 0x20),
            0xE8 => self.add_sp(bus),
            0xE9 => self.jp_hl(bus),
            0xEA => self.ld(bus, Direct8::D, Reg8::A),
            0xEE => self.xor(bus, Imm8),
            0xEF => self.rst(bus, 0x28),

            0xF0 => self.ld(bus, Reg8::A, Direct8::DFF),
            0xF1 => self.pop(bus, Reg16::AF),
            0xF2 => self.ld(bus, Reg8::A, Indirect::CFF),
            0xF3 => self.di(bus),
            0xF5 => self.push(bus, Reg16::AF),
            0xF6 => self.or(bus, Imm8),
            0xF7 => self.rst(bus, 0x30),
            0xF8 => self.ld_hl_sp(bus),
            0xF9 => self.ld_sp_hl(bus),
            0xFA => self.ld(bus, Reg8::A, Direct8::D),
            0xFB => self.ei(bus),
            0xFE => self.cp(bus, Imm8),
            0xFF => self.rst(bus, 0x38),

            _    => panic!("Not opcode '{:02X}' at pc '{:02X}'", self.ctx.opcode, self.regs.pc),
        }
    }

    pub fn cb_prefixed(&mut self, bus: &mut Bus) {
        if let Some(v) = self.read8(bus, Imm8) {
            self.ctx.opcode = v;
            self.ctx.cb = true;
            self.cb_decode(bus);
        }
    }

    pub fn cb_decode(&mut self, bus: &mut Bus) {
        match self.ctx.opcode {
            0x00..=0x05 | 0x07 => self.rlc(bus, self.decode_src()),
            0x06 => self.rlc(bus, Indirect::HL),
            0x08..=0x0D | 0x0F => self.rrc(bus, self.decode_src()),
            0x0E => self.rrc(bus, Indirect::HL),

            0x10..=0x15 | 0x17 => self.rl(bus, self.decode_src()),
            0x16 => self.rl(bus, Indirect::HL),
            0x18..=0x1D | 0x1F => self.rr(bus, self.decode_src()),
            0x1E => self.rr(bus, Indirect::HL),
        
            0x20..=0x25 | 0x27 => self.sla(bus, self.decode_src()),
            0x26 => self.sla(bus, Indirect::HL),
            0x28..=0x2D | 0x2F => self.sra(bus, self.decode_src()),
            0x2E => self.sra(bus, Indirect::HL),

            0x30..=0x35 | 0x37 => self.swap(bus, self.decode_src()),
            0x36 => self.swap(bus, Indirect::HL),
            0x38..=0x3D | 0x3F => self.srl(bus, self.decode_src()),
            0x3E => self.srl(bus, Indirect::HL),

            0x40..=0x45 | 0x47 => self.bit(bus, 0, self.decode_src()),
            0x46 => self.bit(bus, 0, Indirect::HL),
            0x48..=0x4D | 0x4F => self.bit(bus, 1, self.decode_src()),
            0x4E => self.bit(bus, 1, Indirect::HL),

            0x50..=0x55 | 0x57 => self.bit(bus, 2, self.decode_src()),
            0x56 => self.bit(bus, 2, Indirect::HL),
            0x58..=0x5D | 0x5F => self.bit(bus, 3, self.decode_src()),
            0x5E => self.bit(bus, 3, Indirect::HL),

            0x60..=0x65 | 0x67 => self.bit(bus, 4, self.decode_src()),
            0x66 => self.bit(bus, 4, Indirect::HL),
            0x68..=0x6D | 0x6F => self.bit(bus, 5, self.decode_src()),
            0x6E => self.bit(bus, 5, Indirect::HL),

            0x70..=0x75 | 0x77 => self.bit(bus, 6, self.decode_src()),
            0x76 => self.bit(bus, 6, Indirect::HL),
            0x78..=0x7D | 0x7F => self.bit(bus, 7, self.decode_src()),
            0x7E => self.bit(bus, 7, Indirect::HL),

            0x80..=0x85 | 0x87 => self.res(bus, 0, self.decode_src()),
            0x86 => self.res(bus, 0, Indirect::HL),
            0x88..=0x8D | 0x8F => self.res(bus, 1, self.decode_src()),
            0x8E => self.res(bus, 1, Indirect::HL),

            0x90..=0x95 | 0x97 => self.res(bus, 2, self.decode_src()),
            0x96 => self.res(bus, 2, Indirect::HL),
            0x98..=0x9D | 0x9F => self.res(bus, 3, self.decode_src()),
            0x9E => self.res(bus, 3, Indirect::HL),

            0xA0..=0xA5 | 0xA7 => self.res(bus, 4, self.decode_src()),
            0xA6 => self.res(bus, 4, Indirect::HL),
            0xA8..=0xAD | 0xAF => self.res(bus, 5, self.decode_src()),
            0xAE => self.res(bus, 5, Indirect::HL),
            
            0xB0..=0xB5 | 0xB7 => self.res(bus, 6, self.decode_src()),
            0xB6 => self.res(bus, 6, Indirect::HL),
            0xB8..=0xBD | 0xBF => self.res(bus, 7, self.decode_src()),
            0xBE => self.res(bus, 7, Indirect::HL),

            0xC0..=0xC5 | 0xC7 => self.set(bus, 0, self.decode_src()),
            0xC6 => self.set(bus, 0, Indirect::HL),
            0xC8..=0xCD | 0xCF => self.set(bus, 1, self.decode_src()),
            0xCE => self.set(bus, 1, Indirect::HL),

            0xD0..=0xD5 | 0xD7 => self.set(bus, 2, self.decode_src()),
            0xD6 => self.set(bus, 2, Indirect::HL),
            0xD8..=0xDD | 0xDF => self.set(bus, 3, self.decode_src()),
            0xDE => self.set(bus, 3, Indirect::HL),

            0xE0..=0xE5 | 0xE7 => self.set(bus, 4, self.decode_src()),
            0xE6 => self.set(bus, 4, Indirect::HL),
            0xE8..=0xED | 0xEF => self.set(bus, 5, self.decode_src()),
            0xEE => self.set(bus, 5, Indirect::HL),
            
            0xF0..=0xF5 | 0xF7 => self.set(bus, 6, self.decode_src()),
            0xF6 => self.set(bus, 6, Indirect::HL),
            0xF8..=0xFD | 0xFF => self.set(bus, 7, self.decode_src()),
            0xFE => self.set(bus, 7, Indirect::HL),
            _    => panic!("Not opcode {:02x}", self.ctx.opcode),
        }
    }

    fn decode_src(&self) -> Reg8 {
        match self.ctx.opcode & 0b111 {
            0 => Reg8::B,
            1 => Reg8::C,
            2 => Reg8::D,
            3 => Reg8::E,
            4 => Reg8::H,
            5 => Reg8::L,
            6 => todo!("(HL)"),
            7 => Reg8::A,
            _ => unreachable!(),
        }
    }
}
