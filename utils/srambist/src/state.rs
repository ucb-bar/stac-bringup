use crate::pattern::{SramAddr, SramSize, SramWord};

pub struct SramState {
    size: SramSize,
    table: Vec<Option<SramWord>>,
}

impl SramState {
    pub fn new(size: SramSize) -> Self {
        let table = vec![None; size.depth() as usize];
        Self { size, table }
    }

    pub fn read(&self, addr: SramAddr) -> Option<SramWord> {
        assert!(addr < self.size.depth(), "addr out of bounds");
        self.table[addr as usize]
    }

    pub fn write(&mut self, addr: SramAddr, data: SramWord, mask: SramWord) {
        assert!(addr < self.size.depth(), "addr out of bounds");

        let mask_mask = u64::MAX >> (64 - self.size.mask_width());
        if mask == mask_mask {
            self.table[addr as usize] = Some(data);
        } else {
            let entry = self.table[addr as usize]
                .as_mut()
                .expect("cannot perform a partial write on an uninitialized address");
            let mask_gran = self.size.width() / self.size.mask_width();
            for i in 0..self.size.mask_width() {
                if mask & (1 << i) > 0 {
                    let entry_mask = (u64::MAX >> (64 - mask_gran)) << (i * mask_gran);
                    *entry &= !entry_mask;
                    *entry |= entry_mask & data;
                }
            }
        }
    }
}
