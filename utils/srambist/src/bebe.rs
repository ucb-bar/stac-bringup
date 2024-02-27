use crate::executor::Executor;
use crate::pattern::{SramAddr, SramWord};
use std::process::Command;

pub fn bebe_write(addr: u64, data: u64, len: u64) {
    let addr = format!("{addr:X}");
    let data = format!("{data:X}");
    let len = format!("{len}");
    let status = Command::new("sudo")
        .args([
            "python3",
            "bebe_host.py",
            "--no_wait",
            "--quiet",
            "--addr",
            &addr,
            "--wdata",
            &data,
            "--wlen",
            &len,
        ])
        .status()
        .expect("failed to run bebe");
    if !status.success() {
        panic!("bebe exited with non-zero exit code")
    }
}

pub fn bebe_read(addr: u64, len: u64) -> u64 {
    let addr = format!("{addr:X}");
    let len = format!("{len}");
    let output = Command::new("sudo")
        .args([
            "python3",
            "bebe_host.py",
            "--no_wait",
            "--quiet",
            "--addr",
            &addr,
            "--rlen",
            &len,
        ])
        .output()
        .expect("failed to run bebe");
    let output = String::from_utf8(output.stdout).expect("failed to parse bebe output");
    output
        .trim()
        .parse()
        .expect("failed to convert bebe output to u64")
}

pub struct BebeExecutor {
    sram_id: u64,
}

pub struct BebeScratchpadExecutor;

impl BebeExecutor {
    pub fn new(sram_id: u64) -> Self {
        Self { sram_id }
    }
}

impl Executor for BebeExecutor {
    fn init(&mut self) {}
    fn read(&mut self, addr: SramAddr) -> SramWord {
        bebe_write(0x1000, addr as u64, 8);
        // no need to set the mask
        // bebe_write(0x1010, u64::MAX, 8);
        bebe_write(0x1018, 0, 8);
        bebe_write(0x1020, self.sram_id, 8);
        bebe_write(0x1028, 0, 8);
        bebe_write(0x1038, 0, 8);
        bebe_write(0x1180, u64::MAX, 8);
        bebe_read(0x1040, 8)
    }

    fn write(&mut self, addr: SramAddr, data: SramWord, mask: SramWord) {
        bebe_write(0x1000, addr as u64, 8);
        bebe_write(0x1008, data, 8);
        bebe_write(0x1010, mask, 8);
        bebe_write(0x1018, u64::MAX, 8);
        bebe_write(0x1020, self.sram_id, 8);
        bebe_write(0x1028, 0, 8);
        bebe_write(0x1038, 0, 8);
        bebe_write(0x1180, u64::MAX, 8);
    }

    fn finish(&mut self) {}
}

const SCRATCHPAD_BASE_ADDR: u64 = 0x8000000;

impl Executor for BebeScratchpadExecutor {
    fn init(&mut self) {}
    fn read(&mut self, addr: SramAddr) -> SramWord {
        bebe_read(SCRATCHPAD_BASE_ADDR + addr as u64 * 8, 8)
    }

    fn write(&mut self, addr: SramAddr, data: SramWord, mask: SramWord) {
        assert_eq!(mask, 0xFF, "scratchpad only supports mask of all 1s");
        bebe_write(SCRATCHPAD_BASE_ADDR + addr as u64 * 8, data, 8);
    }

    fn finish(&mut self) {}
}
