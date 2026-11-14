use Chuke::{
    audio::Audio, bootrom::Bootrom, bus::Bus, cartridge::Cartridge, cpu::Cpu,
    emulation::step_base_cycle,
};
use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let rom_path = PathBuf::from(args.next().ok_or("Usage: headless ROM [--cgb-bootrom PATH] [--cycles N] [--frames N] [--output PPM] [--expect PPM]")?);
    let mut boot_path = None;
    let mut cycles = 1_000_000u64;
    let mut target_frames = None;
    let mut output = None;
    let mut expected = None;
    while let Some(flag) = args.next() {
        let value = args.next().ok_or("Missing option value")?;
        match flag.to_str() {
            Some("--cgb-bootrom") => boot_path = Some(PathBuf::from(value)),
            Some("--cycles") => cycles = value.to_str().ok_or("Invalid cycle count")?.parse()?,
            Some("--frames") => {
                target_frames = Some(
                    value
                        .to_str()
                        .ok_or("Invalid frame count")?
                        .parse::<u64>()?,
                )
            }
            Some("--output") => output = Some(PathBuf::from(value)),
            Some("--expect") => expected = Some(PathBuf::from(value)),
            _ => return Err("Unknown option".into()),
        }
    }
    if cycles == 0 || target_frames == Some(0) {
        return Err("Counts must be positive".into());
    }
    let boot = match boot_path {
        Some(path) => Bootrom::from_cgb_bytes(fs::read(path)?.into_boxed_slice())?,
        None => Bootrom::new(),
    };
    // Headless verification does not load or overwrite game saves.
    let cartridge = Cartridge::new(fs::read(rom_path)?.into_boxed_slice(), PathBuf::new());
    let mut bus = Bus::new(boot, cartridge, Audio::silent());
    let mut cpu = Cpu::new();
    let mut frames = 0u64;
    let mut elapsed = 0;
    for cycle in 0..cycles {
        elapsed = cycle + 1;
        if step_base_cycle(&mut cpu, &mut bus) {
            frames += 1;
            if target_frames == Some(frames) {
                break;
            }
        }
    }
    let mut ppm = b"P6\n160 144\n255\n".to_vec();
    ppm.extend_from_slice(&bus.ppu.pixel_buffer());
    if let Some(path) = output {
        fs::write(path, &ppm)?;
    }
    println!(
        "base_cycles={elapsed} frames={frames} pc={:04X} a={:02X}",
        cpu.program_counter(),
        cpu.accumulator()
    );
    if target_frames.is_some_and(|target| frames < target) {
        return Err("Frame target not reached before cycle limit".into());
    }
    if let Some(path) = expected {
        if fs::read(path)? != ppm {
            return Err("Screenshot differs from expected PPM".into());
        }
        println!("screenshot_match=true");
    }
    Ok(())
}
