use crate::cpu::{
    Cpu,
    step,
    go,
    operand::{IO8, IO16, Reg16, Imm8, Imm16, Cond},
};
use crate::bus::Bus;

use std::sync::atomic::{AtomicU8, AtomicU16, Ordering::Relaxed};

impl Cpu {
    pub fn nop(&mut self, bus: &Bus) {
        self.fetch(bus);
    }
    pub fn ld<D: Copy, S: Copy>(&mut self, bus: &mut Bus, dst: D, src: S)
    where Self: IO8<D> + IO8<S> {
        step!((), {
            0: if let Some(v) = self.read8(bus, src) {
                VAL8.store(v, Relaxed);
                go!(1);
            },
            1: if self.write8(bus, dst, VAL8.load(Relaxed)).is_some() {
                go!(2);
            },
            2: {
                go!(0);
                self.fetch(bus);
            },
        });
    }
    pub fn ld16<D: Copy, S: Copy>(&mut self, bus: &mut Bus, dst: D, src: S) 
    where Self: IO16<D> + IO16<S> {
        step!((), {
            0: if let Some(v) = self.read16(bus, src) {
                VAL16.store(v, Relaxed);
                go!(1);
            },
            1: if self.write16(bus, dst, VAL16.load(Relaxed)).is_some() {
                go!(2);
            },
            2: {
                go!(0);
                self.fetch(bus);
            },
        });
    }
    
    pub fn cp<S: Copy>(&mut self, bus: &Bus, src: S)
    where Self: IO8<S> {
        if let Some(v) = self.read8(bus, src) {
            let (result, carry) = self.regs.a.overflowing_sub(v);
            self.regs.set_zf(result == 0);
            self.regs.set_nf(true);
            self.regs.set_hf((self.regs.a & 0xf) < (v & 0xf));
            self.regs.set_cf(carry);
            self.fetch(bus);
        }

    }

    pub fn inc<S: Copy>(&mut self, bus: &mut Bus, src: S)
    where Self: IO8<S> {
        step!((), {
            0: if let Some(v) = self.read8(bus, src) {
                let result = v.wrapping_add(1);
                self.regs.set_zf(result == 0);
                self.regs.set_nf(false);
                self.regs.set_hf(v & 0xf == 0xf);
                VAL8.store(result, Relaxed);
                go!(1);
            },
            1: if self.write8(bus, src, VAL8.load(Relaxed)).is_some() {
                go!(0);
                self.fetch(bus);
            },
        });
    }
    pub fn inc16<S: Copy>(&mut self, bus: &mut Bus, src: S)
    where Self: IO16<S> {
        step!((), {
            0: if let Some(v) = self.read16(bus, src) {
                VAL16.store(v.wrapping_add(1), Relaxed);
                go!(1);
            },
            1: if self.write16(bus, src, VAL16.load(Relaxed)).is_some() {
                return go!(2);
            },
            2: {
                go!(0);
                self.fetch(bus);
            },
        });
    }

    pub fn dec<S: Copy>(&mut self, bus: &mut Bus, src : S)
    where Self: IO8<S> {
        step!((), {
            0: if let Some(v) = self.read8(bus, src) {
                let result = v.wrapping_sub(1);
                self.regs.set_zf(result == 0);
                self.regs.set_nf(true);
                self.regs.set_hf(v & 0xf == 0);
                VAL8.store(result, Relaxed);
                go!(1);
            },
            1: if self.write8(bus, src, VAL8.load(Relaxed)).is_some() {
                go!(0);
                self.fetch(bus);
            },
        });
    }
    pub fn dec16<S: Copy>(&mut self, bus: &mut Bus, src: S)
    where Self: IO16<S> {
        step!((), {
            0: if let Some(v) = self.read16(bus, src) {
                VAL16.store(v.wrapping_sub(1), Relaxed);
                go!(1);
            },
            1: if self.write16(bus, src, VAL16.load(Relaxed)).is_some() {
                return go!(2);
            },
            2: {
                go!(0);
                self.fetch(bus);
            },
        });
    }

    pub fn rl<S: Copy>(&mut self, bus: &mut Bus, src: S)
    where Self: IO8<S> {
        step!((), {
            0: if let Some(v) = self.read8(bus, src) {
                let result = (v << 1) | self.regs.cf() as u8;
                self.regs.set_zf(result == 0);
                self.regs.set_nf(false);
                self.regs.set_hf(false);
                self.regs.set_cf(v & 0x80 > 0);
                VAL8.store(result, Relaxed);
                go!(1);
            },
            1: if self.write8(bus, src, VAL8.load(Relaxed)).is_some() {
                go!(0);
                self.fetch(bus);
            },
        });
    }

    pub fn bit<S: Copy>(&mut self, bus: &Bus, bit: usize, src: S)
    where Self: IO8<S> {
        if let Some(mut v) = self.read8(bus, src) {
            v &= 1 << bit;
            self.regs.set_zf(v == 0);
            self.regs.set_nf(false);
            self.regs.set_hf(true);
            self.fetch(bus);
        }
    }

    pub fn push16(&mut self, bus: &mut Bus, val: u16) -> Option<()> {
        step!(None, {
            0: {
                go!(1);
                return None;
            },
            1: {
                let [lo, hi] = u16::to_le_bytes(val);
                self.regs.sp = self.regs.sp.wrapping_sub(1);
                bus.write(&mut self.interrupts, self.regs.sp, hi);
                VAL8.store(lo, Relaxed);
                go!(2);
                return None;
            },
            2: {
                self.regs.sp = self.regs.sp.wrapping_sub(1);
                bus.write(&mut self.interrupts, self.regs.sp, VAL8.load(Relaxed));
                go!(3);
                return None;
            },
            3: return Some(go!(0)),
        });
    }
    pub fn push(&mut self, bus: &mut Bus, src: Reg16) {
        step!((), {
            0: {
                VAL16.store(self.read16(bus, src).unwrap(), Relaxed);
                go!(1);
            },
            1: if self.push16(bus, VAL16.load(Relaxed)).is_some() {
                go!(2);
            },
            2: {
                go!(0);
                self.fetch(bus);
            },
        });
    }

    pub fn pop16(&mut self, bus: &mut Bus) -> Option<u16> {
        step!(None, {
            0: {
                VAL8.store(bus.read(&self.interrupts, self.regs.sp), Relaxed);
                self.regs.sp = self.regs.sp.wrapping_add(1);
                go!(1);
                return None;
            },
            1: {
                let hi = bus.read(&self.interrupts, self.regs.sp);
                self.regs.sp = self.regs.sp.wrapping_add(1);
                VAL16.store(u16::from_le_bytes([VAL8.load(Relaxed), hi]), Relaxed);
                go!(2);
                return None;
            },
            2: {
                go!(0);
                return Some(VAL16.load(Relaxed));
            },
        });
    }
    pub fn pop(&mut self, bus: &mut Bus, dst: Reg16) {
        if let Some(v) = self.pop16(bus) {
            self.write16(bus, dst, v);
            self.fetch(bus);
        }
    }

    fn cond(&mut self, cond: Cond) -> bool {
        match cond {
            Cond::NZ => !self.regs.zf(),
            Cond::Z => self.regs.zf(),
            Cond::NC => !self.regs.cf(),
            Cond::C => self.regs.cf(),
        }
    }
    pub fn jr(&mut self, bus: &Bus) {
        step!((), {
            0: if let Some(v) = self.read8(bus, Imm8) {
                self.regs.pc = self.regs.pc.wrapping_add(v as i8 as u16);
                return go!(1);
            },
            1: {
                go!(0);
                self.fetch(bus);
            } ,
        });
    }
    pub fn jr_c(&mut self, bus: &Bus, c: Cond) {
        step!((), {
            0: if let Some(v) = self.read8(bus, Imm8) {
                go!(1);
                if self.cond(c) {
                    self.regs.pc = self.regs.pc.wrapping_add(v as i8 as u16);
                    return ;
                }
            },
            1: {
                go!(0);
                self.fetch(bus);
            },
        });
    }

    pub fn call(&mut self, bus: &mut Bus) {
        step!((), {
            0: if let Some(v) = self.read16(bus, Imm16) {
                VAL16.store(v, Relaxed);
                go!(1);
            },
            1: if self.push16(bus, self.regs.pc).is_some() {
                self.regs.pc = VAL16.load(Relaxed);
                go!(0);
                self.fetch(bus);
            },
        });
    }

    pub fn ret(&mut self, bus: &mut Bus) {
        step!((), {
            0: if let Some(v) = self.pop16(bus) {
                self.regs.pc = v;
                return go!(1);
            },
            1: {
                go!(0);
                self.fetch(bus);
            },
        });
    }

    pub fn add<S: Copy>(&mut self, bus: &mut Bus, src: S)
    where Self: IO8<S> {
        if let Some(v) = self.read8(bus, src) {
            let (result, carry) = self.regs.a.overflowing_add(v);
            self.regs.set_zf(result == 0);
            self.regs.set_nf(false);
            self.regs.set_hf((self.regs.a & 0xF) + (v & 0xF) > 0xF);
            self.regs.set_cf(carry);
            self.regs.a = result;
            self.fetch(bus);
        }
    }
    pub fn adc<S: Copy>(&mut self, bus: &mut Bus, src: S)
    where Self: IO8<S> {
        if let Some(v) = self.read8(bus, src) {
            let cf = self.regs.cf() as u8;
            let (result, _) = self.regs.a.wrapping_add(v).overflowing_add(cf);
            self.regs.set_zf(result == 0);
            self.regs.set_nf(false);
            self.regs.set_hf((self.regs.a & 0xF) + (v & 0xF) + cf > 0xF);
            self.regs.set_cf(self.regs.a as u16 + v as u16 + cf as u16 > 0xFF);
            self.regs.a = result;
            self.fetch(bus);
        }
    }

    pub fn sub<S: Copy>(&mut self, bus: &mut Bus, src: S)
    where Self: IO8<S> {
        if let Some(v) = self.read8(bus, src) {
            let (result, carry) = self.regs.a.overflowing_sub(v);
            self.regs.set_zf(result == 0);
            self.regs.set_nf(true);
            self.regs.set_hf((self.regs.a & 0xF)  < (v & 0xF));
            self.regs.set_cf(carry);
            self.regs.a = result;
            self.fetch(bus);
        }
    }
    pub fn sbc<S: Copy>(&mut self, bus: &mut Bus, src: S)
    where Self: IO8<S> {
        if let Some(v) = self.read8(bus, src) {
            let cf = self.regs.cf() as u8;
            let (result, _) = self.regs.a.wrapping_sub(v).overflowing_sub(cf);
            self.regs.set_zf(result == 0);
            self.regs.set_nf(true);
            self.regs.set_hf((self.regs.a & 0xF) < (v & 0xF) + cf);
            self.regs.set_cf((self.regs.a as u16) < (v as u16) + cf as u16);
            self.regs.a = result;
            self.fetch(bus);
        }
    }

    pub fn and<S: Copy>(&mut self, bus: &mut Bus, src: S)
    where Self: IO8<S> {
        if let Some(v) = self.read8(bus, src) {
            let result = self.regs.a & v;
            self.regs.set_zf(result == 0);
            self.regs.set_nf(false);
            self.regs.set_hf(true);
            self.regs.set_cf(false);
            self.regs.a = result;
            self.fetch(bus);
        }
    }

    pub fn or<S: Copy>(&mut self, bus: &mut Bus, src: S)
    where Self: IO8<S> {
        if let Some(v) = self.read8(bus, src) {
            let result = self.regs.a | v;
            self.regs.set_zf(result == 0);
            self.regs.set_nf(false);
            self.regs.set_hf(false);
            self.regs.set_cf(false);
            self.regs.a = result;
            self.fetch(bus);
        }
    }

    pub fn xor<S: Copy>(&mut self, bus: &mut Bus, src: S)
    where Self: IO8<S> {
        if let Some(v) = self.read8(bus, src) {
            let result = self.regs.a ^ v;
            self.regs.set_zf(result == 0);
            self.regs.set_nf(false);
            self.regs.set_hf(false);
            self.regs.set_cf(false);
            self.regs.a = result;
            self.fetch(bus);
        }
    }

    pub fn rlca(&mut self, bus: &mut Bus) {
        self.regs.set_zf(false);
        self.regs.set_nf(false);
        self.regs.set_hf(false);
        self.regs.set_cf(( self.regs.a >> 7 ) == 1);
        self.regs.a = ( self.regs.a << 1 ) | ( self.regs.a >> 7 );
        self.fetch(bus);
    }
    pub fn rla(&mut self, bus: &mut Bus) {
        let cf = self.regs.cf() as u8;
        self.regs.set_zf(false);
        self.regs.set_nf(false);
        self.regs.set_hf(false);
        self.regs.set_cf(( self.regs.a >> 7 ) == 1);
        self.regs.a = ( self.regs.a << 1 ) | cf;
        self.fetch(bus);
    }
    pub fn rrca(&mut self, bus: &mut Bus) {
        self.regs.set_zf(false);
        self.regs.set_nf(false);
        self.regs.set_hf(false);
        self.regs.set_cf(( self.regs.a & 1 ) == 1);
        self.regs.a = ( self.regs.a >> 1 ) | ( self.regs.a << 7 );
        self.fetch(bus);  
    }
    pub fn rra(&mut self, bus: &mut Bus) {
        let cf = self.regs.cf() as u8;
        self.regs.set_zf(false);
        self.regs.set_nf(false);
        self.regs.set_hf(false);
        self.regs.set_cf(( self.regs.a & 1 ) == 1);
        self.regs.a = ( self.regs.a >> 1 ) | cf << 7;
        self.fetch(bus);
    }
    pub fn rlc<S: Copy>(&mut self, bus: &mut Bus, src: S)
    where Self: IO8<S> {
        step!((), {
            0: if let Some(v) = self.read8(bus, src) {
                self.regs.set_zf(( v << 1 ) | ( v >> 7 ) == 0);
                self.regs.set_nf(false);
                self.regs.set_hf(false);
                self.regs.set_cf(( v >> 7 ) & 1 == 1);
                VAL8.store(( v << 1 ) | ( v >> 7 ), Relaxed);
                go!(1);
            },
            1: if self.write8(bus, src, VAL8.load(Relaxed)).is_some() {
                go!(0);
                self.fetch(bus);
            },
        });
    }
    pub fn rrc<S: Copy>(&mut self, bus: &mut Bus, src: S) 
    where Self: IO8<S> {
        step!((), {
            0: if let Some(v) = self.read8(bus, src) {
                self.regs.set_zf(( v >> 1 ) | ( v << 7 ) == 0);
                self.regs.set_nf(false);
                self.regs.set_hf(false);
                self.regs.set_cf((v & 1) == 1);
                VAL8.store(( v >> 1 ) | ( v << 7 ), Relaxed);
                go!(1);
            },
            1: if self.write8(bus, src, VAL8.load(Relaxed)).is_some() {
                go!(0);
                self.fetch(bus);
            },
        });
    }
    pub fn rr<S: Copy>(&mut self, bus: &mut Bus, src: S)
    where Self: IO8<S> {
        step!((), {
            0: if let Some(v) = self.read8(bus, src) {
                let cf = self.regs.cf() as u8;
                self.regs.set_zf((( v >> 1 ) | ( cf << 7 )) == 0);
                self.regs.set_nf(false);
                self.regs.set_hf(false);
                self.regs.set_cf(v & 1 == 1);
                VAL8.store(( v >> 1 ) | ( cf << 7 ), Relaxed);
                go!(1);
            },
            1: if self.write8(bus, src, VAL8.load(Relaxed)).is_some() {
                go!(0);
                self.fetch(bus);
            },
        });
    }
    pub fn sla<S: Copy>(&mut self, bus: &mut Bus, src: S) 
    where Self: IO8<S> {
        step!((), {
           0: if let Some(v) = self.read8(bus, src) {
                self.regs.set_zf((v << 1) == 0);
                self.regs.set_nf(false);
                self.regs.set_hf(false);
                self.regs.set_cf((v >> 7) == 1);
                VAL8.store(v << 1, Relaxed);
                go!(1);
           },
           1: if self.write8(bus, src, VAL8.load(Relaxed)).is_some() {
                go!(0);
                self.fetch(bus);
           },
        });
    }
    pub fn sra<S: Copy>(&mut self, bus: &mut Bus, src: S)
    where Self: IO8<S> {
        step!((), {
            0: if let Some(v) = self.read8(bus, src) {
                self.regs.set_zf((v >> 1) == 0);
                self.regs.set_nf(false);
                self.regs.set_hf(false);
                self.regs.set_cf(v & 1 == 1);
                VAL8.store(v >> 1 | (v & 0b_1000_0000), Relaxed);
                go!(1);
            },
            1: if self.write8(bus, src, VAL8.load(Relaxed)).is_some() {
                go!(0);
                self.fetch(bus);
            },
        });
    }
    pub fn srl<S: Copy>(&mut self, bus: &mut Bus, src: S)
    where Self: IO8<S> {
        step!((), {
            0: if let Some(v) = self.read8(bus, src) {
                self.regs.set_zf(v >> 1 == 0);
                self.regs.set_nf(false);
                self.regs.set_hf(false);
                self.regs.set_cf(v & 1 == 1);
                VAL8.store(v >> 1, Relaxed);
                go!(1);
            },
            1: if self.write8(bus, src, VAL8.load(Relaxed)).is_some() {
                go!(0);
                self.fetch(bus);
            },
        });
    }

    pub fn set<S: Copy>(&mut self, bus: &mut Bus, bit: usize, dst: S)
    where Self: IO8<S> {
        step!((), {
            0: if let Some(v) = self.read8(bus, dst) {
                VAL8.store(v | (1 << bit), Relaxed);
                go!(1);
            },
            1: if self.write8(bus, dst, VAL8.load(Relaxed)).is_some() {
                go!(0);
                self.fetch(bus);
            },
        });
    }

    pub fn res<S: Copy>(&mut self, bus: &mut Bus, num: u8, src: S)
    where Self: IO8<S> {
        step!((), {
            0: if let Some(v) = self.read8(bus, src) {
                let result = ( !(1 << num) ) & v;
                VAL8.store(result, Relaxed);
                go!(1);
            },
            1: if self.write8(bus, src, VAL8.load(Relaxed)).is_some() {
                go!(0);
                self.fetch(bus);
            },
        });
    }

    pub fn jp(&mut self, bus: &mut Bus) {
        step!((), {
            0: if let Some(v) = self.read16(bus, Imm16) {
                self.regs.pc = v;
                return go!(1);
            },
            1: {
                go!(0);
                self.fetch(bus);
            },
        });
    }
    pub fn jp_hl(&mut self, bus: &mut Bus) {
        if let Some(v) = self.read16(bus, Reg16::HL) {
            self.regs.pc = v;
        }
        self.fetch(bus);
    }
    pub fn jp_c(&mut self, bus: &mut Bus, c: Cond) {
        step!((), {
            0: if let Some(v) = self.read16(bus, Imm16) {
                go!(1);
                if self.cond(c) {
                    self.regs.pc = v;
                    return ;
                }
            },
            1: {
                go!(0);
                self.fetch(bus);
            },
        });
    }

    pub fn call_c(&mut self, bus: &mut Bus, c: Cond) {
        step!((), {
            0: if let Some(addr) = self.read16(bus, Imm16) {
                VAL16.store(addr, Relaxed);

                if self.cond(c) {
                    go!(1);
                } else {
                    go!(2);
                }
            },
            1: if self.push16(bus, self.regs.pc).is_some() {
                self.regs.pc = VAL16.load(Relaxed);
                go!(2);
            },
            2: {
                go!(0);
                self.fetch(bus);
            },
        });
    }

    pub fn ret_c(&mut self, bus: &mut Bus, c: Cond) {
        step!((), {
            0: return go!(1),
            1: {
                if self.cond(c) {
                    go!(2);
                } else {
                    go!(3);
                }
            },
            2: {
                if let Some(v) = self.pop16(bus) {
                    self.regs.pc = v;
                    return go!(3);
                }
            },
            3: {
                go!(0);
                self.fetch(bus);
            },
        });
    }

    // pub fn call(&mut self, bus: &mut Bus) {
    //     step!((), {
    //         0: if let Some(v) = self.read16(bus, Imm16) {
    //             VAL16.store(v, Relaxed);
    //             go!(1);
    //         },
    //         1: if self.push16(bus, self.regs.pc).is_some() {
    //             self.regs.pc = VAL16.load(Relaxed);
    //             go!(0);
    //             self.fetch(bus);
    //         },
    //     });
    // }

    pub fn rst(&mut self, bus: &mut Bus, addr: u8) {
        if self.push16(bus, self.regs.pc).is_some() {
            self.regs.pc = addr as u16;
            self.fetch(bus);
        }
    }

    pub fn stop(&mut self, bus: &mut Bus) {
        self.regs.pc = self.regs.pc.wrapping_add(1); // STOP's padding byte
        if bus.request_speed_switch() {
            self.resume_fetch = true;
        } else {
            self.fetch(bus);
        }
    }

    pub fn swap<S: Copy>(&mut self, bus: &mut Bus, src: S) 
    where Self: IO8<S> {
        step!((), {
            0: if let Some(v) = self.read8(bus, src) {
                let result = v << 4 | v >> 4;
                VAL8.store(result, Relaxed);
                self.regs.set_zf(result == 0);
                self.regs.set_nf(false);
                self.regs.set_hf(false);
                self.regs.set_cf(false);
                go!(1);
            },
            1: if self.write8(bus, src, VAL8.load(Relaxed)).is_some() {
                go!(0);
                self.fetch(bus);
            },
        });
    }

    pub fn ccf(&mut self, bus: &mut Bus) {
        let carry = self.regs.cf();

        self.regs.set_cf(!carry);
        self.regs.set_nf(false);
        self.regs.set_hf(false);

        self.fetch(bus);
    }

    pub fn scf(&mut self, bus: &mut Bus) {
        self.regs.set_nf(false);
        self.regs.set_hf(false);
        self.regs.set_cf(true);
        self.fetch(bus);
    }

    pub fn daa(&mut self, bus: &Bus) {
        let mut correction = 0;
        let mut cf = false;
        if self.regs.cf() || (!self.regs.nf() && self.regs.a > 0x99) {
            cf = true;
            correction |= 0x60;
        }
        if self.regs.hf() || (!self.regs.nf() && (self.regs.a & 0x0f > 0x09)) {
            correction |= 0x06;
        }
        if self.regs.nf() {
            self.regs.a = self.regs.a.wrapping_sub(correction);
        } else {
            self.regs.a = self.regs.a.wrapping_add(correction);
        }
        self.regs.set_zf(self.regs.a == 0);
        self.regs.set_hf(false);
        self.regs.set_cf(cf);
        self.fetch(bus);
    }

    pub fn cpl(&mut self, bus: &mut Bus) {
        self.regs.a = (!self.regs.a) & 0xFF;
        self.regs.set_nf(true);
        self.regs.set_hf(true);
        self.fetch(bus);
    }

    pub fn ld_sp_hl(&mut self, bus: &mut Bus) {
        step!((), {
            0: {
                return go!(1);
            },
            1: if let Some(v) = self.read16(bus, Reg16::HL) {
                self.regs.sp = v;
                go!(0);
                self.fetch(bus);
            },
        });
    }

    pub fn ld_hl_sp(&mut self, bus: &mut Bus) {
        step!((), {
            0: {
                return go!(1);
            },
            1: if let Some(v) = self.read8(bus, Imm8) {
                let (result, _) = self.read16(bus, Reg16::SP).unwrap().overflowing_add(v as i8 as u16);
                self.regs.set_zf(false);
                self.regs.set_nf(false);
                self.regs.set_hf((self.regs.sp & 0xF) + (v as i8 as u16 & 0xF) > 0xF);
                self.regs.set_cf((self.regs.sp & 0xFF) + (v as i8 as u16 & 0xFF) > 0xFF);
                self.write16(bus, Reg16::HL, result);
                go!(0);
                self.fetch(bus);
            },
        });
    }

    pub fn add_hl_reg16(&mut self, bus: &mut Bus, src: Reg16) {
        step!((), {
            0: {
                return go!(1);
            },
            1: if let Some(v) = self.read16(bus, src){
                let (result, carry) = self.regs.hl().overflowing_add(v);
                self.regs.set_nf(false);
                self.regs.set_hf((self.regs.hl() & 0xFFF) + (v & 0xFFF) > 0xFFF);
                self.regs.set_cf(carry);
                self.write16(bus, Reg16::HL, result);
                go!(0);
                self.fetch(bus);
            },
        });
    }

    pub fn add_sp(&mut self, bus: &mut Bus) {
        step!((), {
            0: if let Some(v) = self.read8(bus, Imm8) {
                let (result, _) = self.regs.sp.overflowing_add(v as i8 as u16);
                self.regs.set_zf(false);
                self.regs.set_nf(false);
                self.regs.set_hf((self.regs.sp & 0xF) + (v as i8 as u16 & 0xF) > 0xF);
                self.regs.set_cf((self.regs.sp & 0xFF) + (v as i8 as u16 & 0xFF) > 0xFF);
                self.regs.sp = result;
                return go!(1);
            },
            1:  {
                return go!(2);
            },
            2: {
                go!(0);
                self.fetch(bus);
            },
        });
    }

    // interrupts
    pub fn reti(&mut self, bus: &mut Bus) {
        step!((), {
            0: if let Some(v) = self.pop16(bus) {
                self.regs.pc = v;
                return go!(1);
            },
            1: {
                self.interrupts.ime = true;
                go!(0);
                self.fetch(bus);
            },
        });
    }
    pub fn ei(&mut self, bus: &Bus) {
        self.fetch(bus);
        self.interrupts.ime = true;
    }
    pub fn di(&mut self, bus: &Bus) {
        self.interrupts.ime = false;
        self.fetch(bus);
    }
    pub fn halt(&mut self, bus: &Bus) {
        step!((), {
            0: if self.interrupts.get_interrupt() > 0 {
                self.fetch(bus);
            } else {
                return go!(1);
            },
            1: {
                if self.interrupts.get_interrupt() > 0 {
                    go!(0);
                    self.fetch(bus);
                }
            },
        });
    }
}
