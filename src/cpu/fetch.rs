use crate::cpu::Cpu;
use crate::bus::Bus;

impl Cpu {
    pub fn fetch(&mut self, bus: &Bus) {
        // print!("pc: {:x}, ", self.regs.pc);
        self.ctx.opcode = bus.read(&self.interrupts,self.regs.pc);
        // println!("opcode : {:x}", self.ctx.opcode);
        if self.interrupts.ime && self.interrupts.get_interrupt() > 0 {
            self.ctx.int = true;
        } else {
            self.regs.pc = self.regs.pc.wrapping_add(1);
            self.ctx.int = false;
        }
        self.ctx.cb = false;
    }
}
