use crate::{bus::Bus, cpu::Cpu};

/// Advance four PPU dots (one normal-speed M-cycle), independent of host output.
pub fn step_base_cycle(cpu: &mut Cpu, bus: &mut Bus) -> bool {
    let cpu_cycles = bus.speed.cpu_cycles_per_base_cycle();
    for _ in 0..cpu_cycles {
        if bus.speed.advance_switch() { continue; }
        let dma_stalls = !bus.hdma_waits_for_cpu(cpu.is_halted())
            && bus.step_vram_dma(&cpu.interrupts);
        if !dma_stalls { cpu.emulate_cycle(bus); }
        if bus.speed.is_switching() { continue; }
        bus.timer.emulate_cycle(&mut cpu.interrupts);
        if let Some(addr) = bus.ppu.oam_dma {
            bus.ppu.oam_dma_emulate_cycle(bus.read(&cpu.interrupts, addr));
        }
    }
    bus.apu.emulate_cycle();
    bus.ppu.emulate_cycle(&mut cpu.interrupts)
}

#[cfg(test)]
mod tests {
    use crate::{audio::Audio, bootrom::Bootrom, bus::Bus, cartridge::Cartridge, cpu::Cpu};

    #[test]
    fn synthetic_cgb_boot_hands_off_and_runs_without_sdl() {
        let mut rom = vec![0; 0x8000];
        rom[0x100..0x10D].copy_from_slice(&[0x31, 0xFE, 0xFF, 0x3E, 1, 0xE0, 0x4D, 0x10, 0, 0x3E, 0x5A, 0x18, 0xFE]);
        rom[0x40] = 0xD9; // RETI
        rom[0x143] = 0x80;
        rom[0x14D] = rom[0x134..=0x14C].iter().fold(0u8, |sum, byte| sum.wrapping_sub(*byte).wrapping_sub(1));
        let cartridge = Cartridge::new(rom.into_boxed_slice(), std::env::temp_dir().join("chuke-synthetic.sav"));
        let mut boot = vec![0; 0x900];
        boot[..3].copy_from_slice(&[0xC3, 0xFC, 0]);
        boot[0xFC..0x100].copy_from_slice(&[0x3E, 0x11, 0xE0, 0x50]);
        let mut bus = Bus::new(Bootrom::from_cgb_bytes(boot.into_boxed_slice()).unwrap(), cartridge, Audio::silent());
        let mut cpu = Cpu::new();
        for _ in 0..64 {
            super::step_base_cycle(&mut cpu, &mut bus);
            if bus.speed.is_switching() { break; }
        }
        assert!(bus.speed.is_switching());
        assert_eq!(cpu.program_counter(), 0x109);
        cpu.interrupts.ime = true;
        cpu.interrupts.int_enable = 1;
        cpu.interrupts.int_flags = 1;
        for _ in 0..2050 { super::step_base_cycle(&mut cpu, &mut bus); }
        assert!(bus.speed.is_double());
        assert_eq!(cpu.program_counter(), 0x109);
        assert_eq!(bus.timer.read(0xFF04), 0);
        for _ in 0..32 { super::step_base_cycle(&mut cpu, &mut bus); }
        assert_eq!(cpu.accumulator(), 0x5A);
        assert_eq!(cpu.interrupts.int_flags & 1, 0);

        // Eight GDMA blocks stall 128 fast CPU cycles = 64 base cycles.
        bus.write(&mut cpu.interrupts, 0xFF51, 0xC0);
        bus.write(&mut cpu.interrupts, 0xFF55, 7);
        let pc = cpu.program_counter();
        let div = bus.timer.read(0xFF04);
        for _ in 0..64 { super::step_base_cycle(&mut cpu, &mut bus); }
        assert_eq!(cpu.program_counter(), pc);
        assert_eq!(bus.read(&cpu.interrupts, 0xFF55), 0xFF);
        assert_eq!(bus.timer.read(0xFF04).wrapping_sub(div), 2);

        bus.write(&mut cpu.interrupts, 0xFF40, 0x91);
        for _ in 0..114 { super::step_base_cycle(&mut cpu, &mut bus); }
        assert_eq!(bus.ppu.read(0xFF44), 1);
        bus.write(&mut cpu.interrupts, 0xFF51, 0xC0);
        bus.write(&mut cpu.interrupts, 0xFF52, 0x80);
        bus.write(&mut cpu.interrupts, 0xFF55, 0x82);
        for _ in 0..71 { super::step_base_cycle(&mut cpu, &mut bus); }
        assert_eq!(bus.read(&cpu.interrupts, 0xFF55), 1);
        for _ in 0..114 { super::step_base_cycle(&mut cpu, &mut bus); }
        assert_eq!(bus.read(&cpu.interrupts, 0xFF55), 0);
        for _ in 0..114 { super::step_base_cycle(&mut cpu, &mut bus); }
        assert_eq!(bus.read(&cpu.interrupts, 0xFF55), 0xFF);
        for _ in 0..17_556 {
            if bus.ppu.read(0xFF44) == 144 { break; }
            super::step_base_cycle(&mut cpu, &mut bus);
        }
        assert_eq!(bus.ppu.read(0xFF44), 144);
        bus.write(&mut cpu.interrupts, 0xFF55, 0x80);
        for _ in 0..114 * 9 { super::step_base_cycle(&mut cpu, &mut bus); }
        assert_eq!(bus.read(&cpu.interrupts, 0xFF55), 0);
    }
}
