use crate::bebe::{bebe_read, bebe_write};

mod consts {
    pub const BASE: u64 = 0x1000;
    pub const ADDR: u64 = 0x0 + BASE;
    pub const DIN: u64 = 0x8 + BASE;
    pub const MASK: u64 = 0x10 + BASE;
    pub const WE: u64 = 0x18 + BASE;
    pub const SRAM_ID: u64 = 0x20 + BASE;
    pub const SRAM_SEL: u64 = 0x28 + BASE;
    pub const SAE_CTL: u64 = 0x30 + BASE;
    pub const SAE_SEL: u64 = 0x38 + BASE;
    pub const DOUT: u64 = 0x40 + BASE;
    pub const TDC: u64 = 0x48 + BASE;
    pub const DONE: u64 = 0x68 + BASE;
    pub const BIST_RAND_SEED: u64 = 0x70 + BASE;
    pub const BIST_SIG_SEED: u64 = 0x80 + BASE;
    pub const BIST_MAX_ROW_ADDR: u64 = 0x88 + BASE;
    pub const BIST_MAX_COL_ADDR: u64 = 0x90 + BASE;
    pub const BIST_INNER_DIM: u64 = 0x98 + BASE;
    pub const BIST_ELEMENT_SEQUENCE: u64 = 0xa0 + BASE;
    pub const BIST_PATTERN_TABLE: u64 = 0x120 + BASE;
    pub const BIST_MAX_ELEMENT_IDX: u64 = 0x140 + BASE;
    pub const BIST_CYCLE_LIMIT: u64 = 0x148 + BASE;
    pub const BIST_STOP_ON_FAILURE: u64 = 0x150 + BASE;
    pub const BIST_FAIL: u64 = 0x158 + BASE;
    pub const BIST_FAIL_CYCLE: u64 = 0x160 + BASE;
    pub const BIST_EXPECTED: u64 = 0x168 + BASE;
    pub const BIST_RECEIVED: u64 = 0x170 + BASE;
    pub const BIST_SIGNATURE: u64 = 0x178 + BASE;
    pub const EX: u64 = 0x180 + BASE;
}

pub fn read_sram(id: u64, addr: u64) -> u64 {
    bebe_write(consts::ADDR, addr, 8);
    bebe_write(consts::WE, 0, 8);
    bebe_write(consts::EX, 1, 8);
    bebe_read(consts::DOUT, 4)
}

pub fn write_sram(id: u64, addr: u64, data: u64) {
    bebe_write(consts::ADDR, addr, 8);
    bebe_write(consts::DIN, data, 8);
    bebe_write(consts::MASK, u64::MAX, 8);
    bebe_write(consts::WE, 1, 8);
    bebe_write(consts::EX, 1, 8);
}

pub fn sweep_tdc_test(id: u64, tdc_min: u64, tdc_max: u64) {
    assert!(tdc_min <= tdc_max);

    bebe_write(consts::SRAM_ID, id, 8);
    bebe_write(consts::SRAM_SEL, 0, 8);
    bebe_write(consts::SAE_SEL, 2, 8);

    for code in tdc_min..=tdc_max {
        bebe_write(consts::SAE_CTL, code, 8);

        let c1 = 0xdeadbeef;
        let c2 = 0x932a39b1;
        let c3 = 0x8939471a;
        let c4 = 0x29401949;

        write_sram(id, 0, c1);
        write_sram(id, 1, c2);
        write_sram(id, 2, c3);
        write_sram(id, 3, c4);

        let pass = read_sram(id, 0) == c1
            && read_sram(id, 1) == c2
            && read_sram(id, 2) == c3
            && read_sram(id, 3) == c4;
        if !pass {
            println!("tdc code {code} passed!");
        } else {
            println!("tdc code {code} failed, trying next code");
        }
    }
}
