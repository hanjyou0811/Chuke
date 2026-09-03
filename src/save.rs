use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

const QUIET_FRAMES: u64 = 15;
const MAX_DIRTY_FRAMES: u64 = 300;

pub struct SaveManager {
    path: PathBuf,
    enabled: bool,
    dirty: bool,
    wrote_this_frame: bool,
    last_write_frame: u64,
    last_save_frame: u64,
}

impl SaveManager {
    pub fn new(path: PathBuf, enabled: bool) -> Self {
        Self {
            path,
            enabled,
            dirty: false,
            wrote_this_frame: false,
            last_write_frame: 0,
            last_save_frame: 0,
        }
    }

    pub fn load(&self, ram: &mut [u8]) -> io::Result<()> {
        if !self.enabled || ram.is_empty() {
            return Ok(());
        }

        let data = match fs::read(&self.path) {
            Ok(data) => data,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };

        let copy_len = data.len().min(ram.len());
        ram[..copy_len].copy_from_slice(&data[..copy_len]);
        Ok(())
    }

    pub fn mark_dirty(&mut self) {
        if self.enabled {
            self.dirty = true;
            self.wrote_this_frame = true;
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.enabled && self.dirty
    }

    pub fn save_ready(&mut self, current_frame: u64) -> bool {
        if !self.is_dirty() {
            self.wrote_this_frame = false;
            return false;
        }

        if self.wrote_this_frame {
            self.last_write_frame = current_frame;
            self.wrote_this_frame = false;
        }

        current_frame.saturating_sub(self.last_write_frame) >= QUIET_FRAMES
            || current_frame.saturating_sub(self.last_save_frame) >= MAX_DIRTY_FRAMES
    }

    pub fn save(&mut self, ram: &[u8], current_frame: u64) -> io::Result<()> {
        if !self.is_dirty() {
            return Ok(());
        }

        save_atomic(&self.path, ram)?;
        self.dirty = false;
        self.wrote_this_frame = false;
        self.last_save_frame = current_frame;
        Ok(())
    }
}

fn save_atomic(path: &Path, data: &[u8]) -> io::Result<()> {
    let temp_path = path.with_extension("sav.tmp");
    {
        let mut file = File::create(&temp_path)?;
        file.write_all(data)?;
        file.sync_all()?;
    }
    fs::rename(temp_path, path)
}