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
    // The selected boot ROM determines the hardware, independently of the game.
    let mut options = env::args_os().skip(2);
    let bootrom = match options.next() {
        None => Bootrom::new(),
        Some(flag) if flag == "--cgb-bootrom" => {
            let boot_path = options.next().ok_or("--cgb-bootrom requires a file path")?;
            let data = read_file(Path::new(&boot_path))?;
            Bootrom::from_cgb_bytes(data)?
        }
        Some(_) => return Err("Usage: Chuke ROM [--cgb-bootrom PATH]".into()),
    };
    if options.next().is_some() {
        return Err("Usage: Chuke ROM [--cgb-bootrom PATH]".into());
    }
    let mut gameboy = GameBoy::new(bootrom, cartridge);
    gameboy.run()?;

    Ok(())
}
