#[derive(Default)]
pub struct Speed {
    double: bool,
    armed: bool,
    switching: u16,
}

impl Speed {
    pub fn read(&self) -> u8 { 0x7E | ((self.double as u8) << 7) | self.armed as u8 }
    pub fn write(&mut self, value: u8) { self.armed = value & 1 != 0; }
    pub fn cpu_cycles_per_base_cycle(&self) -> usize { if self.double { 2 } else { 1 } }
    pub fn is_switching(&self) -> bool { self.switching != 0 }
    pub fn is_double(&self) -> bool { self.double }
    pub fn request_switch(&mut self) -> bool {
        if !self.armed || self.switching != 0 { return false; }
        self.armed = false;
        self.switching = 2050;
        true
    }
    /// Called once per CPU M-cycle; true means CPU and DIV remain stopped.
    pub fn advance_switch(&mut self) -> bool {
        if self.switching == 0 { return false; }
        self.switching -= 1;
        if self.switching == 0 { self.double = !self.double; }
        true
    }
}
