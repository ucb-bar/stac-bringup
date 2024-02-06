use crate::pattern::{FixedPattern, FixedSramOp, SramAddr, SramSize, SramWord};
use crate::state::SramState;

pub trait Executor {
    fn init(&mut self);
    fn read(&mut self, addr: SramAddr) -> SramWord;
    fn write(&mut self, addr: SramAddr, data: SramWord, mask: SramWord);

    fn finish(&mut self);
}

pub struct IdealExecutor {
    state: SramState,
}

impl IdealExecutor {
    pub fn new(size: SramSize) -> Self {
        let state = SramState::new(size);
        Self { state }
    }
}

impl Executor for IdealExecutor {
    fn init(&mut self) {}

    fn read(&mut self, addr: SramAddr) -> SramWord {
        self.state
            .read(addr)
            .expect("tried to read from an uninitialized address")
    }

    fn write(&mut self, addr: SramAddr, data: SramWord, mask: SramWord) {
        self.state.write(addr, data, mask);
    }

    fn finish(&mut self) {}
}

/// A collection of all errors produced by executing a test.
pub struct TestPatternErrors {
    pub errors: Vec<BistError>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct BistError {
    op: usize,
    expected: SramWord,
    received: SramWord,
}
pub fn execute<E: Executor>(pattern: FixedPattern, mut ex: E) -> Result<(), BistError> {
    println!("Beginning SRAM BIST test");

    ex.init();
    let res = execute_inner(pattern, &mut ex);
    ex.finish();

    res
}

fn execute_inner<E: Executor>(pattern: FixedPattern, ex: &mut E) -> Result<(), TestPatternErrors> {
    let mut errors = Vec::new();
    for (i, op) in pattern.ops().enumerate() {
        match op {
            FixedSramOp::Read { data, addr } => {
                print!("Reading {addr:#x}...\t");
                let dout = ex.read(addr);
                if dout == data {
                    println!("OK");
                } else {
                    println!("ERROR: got {dout:#x}, expected {data:#x}");
                    errors.push(BistError {
                        op: i,
                        expected: data,
                        received: dout,
                    });
                }
            }
            FixedSramOp::Write { data, addr, mask } => {
                print!("Writing {addr:#x} with data = {data:#x}, mask = {mask:#x}...\t");
                ex.write(addr, data, mask);
                println!("DONE");
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(TestPatternErrors {
            errors,
        })
    }
}
