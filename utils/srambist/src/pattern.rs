use crate::state::SramState;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use serde::{Deserialize, Serialize};

pub type SramWord = u64;
pub type SramAddr = u32;

/// A [`Pattern`] is a general pattern that may be applied
/// to various sizes of SRAM.

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Pattern {
    elements: Vec<Element>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Element {
    ops: Vec<SramOp>,
    addr_seq: AddrSeq,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum AddrSeq {
    Up,
    Down,
    /// Try the given number of random addresses
    Rand(u64),
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum SramOp {
    Read,
    Write { data: SramInput, mask: SramInput },
    Rand { mask: RandMask },
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum RandMask {
    Fixed(SramWord),
    Rand,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum SramInput {
    Fixed(SramWord),
    Rand,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct SramSize {
    width: SramWord,
    depth: SramAddr,
    mask_width: SramWord,
}

impl Pattern {
    pub fn mats_plus() -> Self {
        Self {
            elements: vec![
                Element {
                    addr_seq: AddrSeq::Up,
                    ops: vec![SramOp::Write {
                        data: SramInput::Fixed(0),
                        mask: SramInput::Fixed(u64::MAX),
                    }],
                },
                Element {
                    addr_seq: AddrSeq::Up,
                    ops: vec![
                        SramOp::Read,
                        SramOp::Write {
                            data: SramInput::Fixed(u64::MAX),
                            mask: SramInput::Fixed(u64::MAX),
                        },
                    ],
                },
                Element {
                    addr_seq: AddrSeq::Down,
                    ops: vec![
                        SramOp::Read,
                        SramOp::Write {
                            data: SramInput::Fixed(0),
                            mask: SramInput::Fixed(u64::MAX),
                        },
                    ],
                },
            ],
        }
    }

    pub fn march_cm() -> Self {
        Self {
            elements: vec![
                Element {
                    addr_seq: AddrSeq::Up,
                    ops: vec![SramOp::Write {
                        data: SramInput::Fixed(0),
                        mask: SramInput::Fixed(u64::MAX),
                    }],
                },
                Element {
                    addr_seq: AddrSeq::Up,
                    ops: vec![
                        SramOp::Read,
                        SramOp::Write {
                            data: SramInput::Fixed(u64::MAX),
                            mask: SramInput::Fixed(u64::MAX),
                        },
                    ],
                },
                Element {
                    addr_seq: AddrSeq::Up,
                    ops: vec![
                        SramOp::Read,
                        SramOp::Write {
                            data: SramInput::Fixed(0),
                            mask: SramInput::Fixed(u64::MAX),
                        },
                    ],
                },
                Element {
                    addr_seq: AddrSeq::Down,
                    ops: vec![
                        SramOp::Read,
                        SramOp::Write {
                            data: SramInput::Fixed(u64::MAX),
                            mask: SramInput::Fixed(u64::MAX),
                        },
                    ],
                },
                Element {
                    addr_seq: AddrSeq::Down,
                    ops: vec![
                        SramOp::Read,
                        SramOp::Write {
                            data: SramInput::Fixed(0),
                            mask: SramInput::Fixed(u64::MAX),
                        },
                    ],
                },
                Element {
                    addr_seq: AddrSeq::Up,
                    ops: vec![SramOp::Read],
                },
            ],
        }
    }

    pub fn rand(n: u64) -> Self {
        Self {
            elements: vec![
                Element {
                    addr_seq: AddrSeq::Up,
                    ops: vec![
                        SramOp::Write {
                            data: SramInput::Fixed(0),
                            mask: SramInput::Fixed(u64::MAX),
                        },
                        SramOp::Read,
                    ],
                },
                Element {
                    addr_seq: AddrSeq::Rand(n),
                    ops: vec![SramOp::Rand {
                        mask: RandMask::Fixed(u64::MAX),
                    }],
                },
            ],
        }
    }
}

impl SramSize {
    pub fn new(width: SramWord, depth: SramAddr, mask_width: SramWord) -> Self {
        assert!(width > 0, "width must be greater than 0");
        assert!(depth > 0, "depth must be greater than 0");
        assert!(mask_width > 0, "mask width must be greater than 0");
        assert_eq!(
            width % mask_width,
            0,
            "SRAM width must be an even multiple of mask width"
        );
        Self {
            width,
            depth,
            mask_width,
        }
    }

    pub fn width(&self) -> SramWord {
        self.width
    }
    pub fn depth(&self) -> SramAddr {
        self.depth
    }
    pub fn mask_width(&self) -> SramWord {
        self.mask_width
    }
}

pub enum FixedSramOp {
    Read {
        addr: SramAddr,
        data: SramWord,
    },
    Write {
        addr: SramAddr,
        data: SramWord,
        mask: SramWord,
    },
}

/// A pattern adapted to be specific to a particular SRAM size,
/// with all randomness removed by the use of a random seed.
pub struct FixedPattern {
    pattern: Pattern,
    size: SramSize,
    seed: u64,
}

impl FixedPattern {
    pub fn new(pattern: Pattern, size: SramSize, seed: u64) -> Self {
        Self {
            pattern,
            size,
            seed,
        }
    }

    pub fn ops(&self) -> impl Iterator<Item = FixedSramOp> {
        let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(self.seed);
        let mut state = SramState::new(self.size);
        let mut ops = Vec::new();
        let dmask = u64::MAX >> (64 - self.size.width);
        let mask_mask = u64::MAX >> (64 - self.size.mask_width);
        for elt in self.pattern.elements.iter() {
            let addrs: Vec<u32> = match elt.addr_seq {
                AddrSeq::Up => (0..self.size.depth).collect(),
                AddrSeq::Down => (0..self.size.depth).rev().collect(),
                AddrSeq::Rand(n) => (0..n)
                    .map(|_| (rng.next_u64() % self.size.depth as u64) as u32)
                    .collect(),
            };

            for addr in addrs {
                for op in elt.ops.iter() {
                    let op = match op {
                        SramOp::Read => FixedSramOp::Read {
                            addr,
                            data: state
                                .read(addr)
                                .expect("pattern attempted to read from an uninitialized address"),
                        },
                        SramOp::Write { data, mask } => FixedSramOp::Write {
                            addr,
                            data: match data {
                                SramInput::Fixed(data) => *data & dmask,
                                SramInput::Rand => rng.next_u64() & dmask,
                            },
                            mask: match mask {
                                SramInput::Fixed(mask) => *mask & mask_mask,
                                SramInput::Rand => rng.next_u64() & mask_mask,
                            },
                        },
                        SramOp::Rand { mask } => {
                            if rng.next_u32() & 1 > 0 {
                                FixedSramOp::Read {
                                    addr,
                                    data: state.read(addr).expect(
                                        "pattern attempted to read from an uninitialized address",
                                    ),
                                }
                            } else {
                                let mask = match mask {
                                    RandMask::Fixed(mask) => *mask & mask_mask,
                                    RandMask::Rand => rng.next_u64() & mask_mask,
                                };

                                FixedSramOp::Write {
                                    addr,
                                    data: rng.next_u64() & dmask,
                                    mask,
                                }
                            }
                        }
                    };

                    if let FixedSramOp::Write { data, addr, mask } = op {
                        state.write(addr, data, mask);
                    }
                    ops.push(op);
                }
            }
        }

        ops.into_iter()
    }
}
