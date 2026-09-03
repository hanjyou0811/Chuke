use Chuke::{
    bootrom::Bootrom, cartridge::Cartridge, gameboy::GameBoy,
};

use std::{
    env,
    fs,
    io,
    path::{Path, PathBuf},
};

fn get_file_path() -> Result<PathBuf, &'static str> {
    env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("Need ROM file...")
}

fn read_file(path: &Path) -> io::Result<Box<[u8]>> {
    let data: Vec<u8> = fs::read(path)?;
    Ok(data.into_boxed_slice())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = get_file_path()?;
    let cartridge_data: Box<[u8]> = read_file(&path)?;
    let save_path = path.with_extension("sav");
    let cartridge = Cartridge::new(cartridge_data, save_path);
    let bootrom = Bootrom::new(cartridge.is_cgb());
    let mut gameboy = GameBoy::new(bootrom, cartridge);
    gameboy.run()?;

    Ok(())
}
