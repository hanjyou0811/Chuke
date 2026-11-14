mod registers;
mod fetch;
mod decode;
mod instructions;
mod operand;
pub mod interrupts;

use crate::cpu::{
    registers::Registers,
    interrupts::{
        Interrupts,
        VBLANK,
        STAT,
        TIMER,
        SERIAL,
        JOYPAD,
    },
};

use std::sync::atomic::{
    AtomicU8, 
    AtomicU16,
    Ordering::Relaxed,
};
use crate::bus::Bus;

macro_rules! step {
  ($d:expr, {$($c:tt : $e:expr,)*}) => {
    static STEP: AtomicU8 = AtomicU8::new(0);
    #[allow(dead_code)]
    static VAL8: AtomicU8 = AtomicU8::new(0);
    #[allow(dead_code)]
    static VAL16: AtomicU16 = AtomicU16::new(0);
    $(if STEP.load(Relaxed) == $c { $e })* else { return $d; }
  };
}
pub(crate) use step;
macro_rules! go {
    ($e:expr) => {
        STEP.store($e, Relaxed)
    };
}
pub(crate) use go;

#[derive(Default)]
struct Ctx {
    opcode: u8,
    cb: bool,
    int: bool, 
}
pub struct Cpu {
    resume_fetch: bool,
    regs: Registers,
    ctx: Ctx,
    pub interrupts: Interrupts,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            resume_fetch: false,
            regs: Registers::default(),
            ctx: Ctx::default(),
            interrupts: Interrupts::default(),
        }
    } 
    pub fn is_halted(&self) -> bool {
        !self.ctx.cb && !self.ctx.int && self.ctx.opcode == 0x76 && self.interrupts.get_interrupt() == 0
    }

    pub fn emulate_cycle(&mut self, bus: &mut Bus) {
        if self.resume_fetch {
            self.resume_fetch = false;
            self.fetch(bus);
            return;
        }
        if self.ctx.int {
            self.call_isr(bus); // 割り込み処理
        } else {
            self.decode(bus);
        }
    }
    fn call_isr(&mut self, bus: &mut Bus) {
        step!((), {
            0: if let Some(_) = self.push16(bus, self.regs.pc) {
                let highest_int: u8 = 1 << self.interrupts.get_interrupt().trailing_zeros();
                self.interrupts.int_flags &= !highest_int;
                self.regs.pc = match highest_int {
                    VBLANK => 0x0040,
                    STAT   => 0x0048,
                    TIMER  => 0x0050,
                    SERIAL => 0x0058,
                    JOYPAD => 0x0060,
                    _      => panic!("Invalid interrupt : {:02x}", highest_int),
                };
                return go!(1);
            },
            1: {
                self.interrupts.ime = false;
                go!(0);
                self.fetch(bus);
            },
        });
    }
}

#[cfg(test)]
mod halt_state_tests {
    use super::*;

    #[test]
    fn cb76_is_not_halt() {
        let mut cpu = Cpu::new();
        cpu.ctx.opcode = 0x76;
        assert!(cpu.is_halted());
        cpu.ctx.cb = true;
        assert!(!cpu.is_halted());
        cpu.ctx.cb = false;
        cpu.ctx.int = true;
        assert!(!cpu.is_halted());
    }
}
