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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key1_arms_stop_and_switches_after_delay() {
        let mut speed = Speed::default();
        assert_eq!(speed.read(), 0x7E);
        assert!(!speed.request_switch());
        speed.write(0xFF);
        assert_eq!(speed.read(), 0x7F);
        assert!(speed.request_switch());
        for _ in 0..2050 { assert!(speed.advance_switch()); }
        assert!(!speed.advance_switch());
        assert_eq!(speed.read(), 0xFE);
        assert_eq!(speed.cpu_cycles_per_base_cycle(), 2);
        speed.write(1);
        assert!(speed.request_switch());
        for _ in 0..2050 { speed.advance_switch(); }
        assert_eq!(speed.read(), 0x7E);
        assert_eq!(speed.cpu_cycles_per_base_cycle(), 1);
    }
}
