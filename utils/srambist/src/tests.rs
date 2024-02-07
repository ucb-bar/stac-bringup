use crate::bebe::BebeScratchpadExecutor;
use crate::executor::{execute, IdealExecutor};
use crate::pattern::{FixedPattern, Pattern, SramSize};

/// The size of the scratchpad on the STAC-V1 test chip.
const STAC_SCRATCHPAD_SIZE: SramSize = SramSize {
    width: 64,
    depth: 512,
    mask_width: 8,
};

#[test]
fn mats_plus_ideal_executor() {
    let size = SramSize::new(32, 256, 4);
    let ex = IdealExecutor::new(size);
    let pat = FixedPattern::new(Pattern::mats_plus(), size, 1);
    execute(pat, ex).expect("MATS+ pattern should execute correctly with an ideal executor");
}

#[test]
fn march_cm_ideal_executor() {
    let size = SramSize::new(32, 256, 4);
    let ex = IdealExecutor::new(size);
    let pat = FixedPattern::new(Pattern::march_cm(), size, 1);
    execute(pat, ex).expect("March C- pattern should execute correctly with an ideal executor");
}

#[test]
fn rand4096_ideal_executor() {
    let size = SramSize::new(32, 256, 4);
    let ex = IdealExecutor::new(size);
    let pat = FixedPattern::new(Pattern::rand(4096), size, 1);
    execute(pat, ex).expect("Rand 4096 pattern should execute correctly with an ideal executor");
}

#[test]
#[ignore = "requires test chip"]
fn mats_plus_bebe_scratchpad() {
    let size = STAC_SCRATCHPAD_SIZE;
    let ex = BebeScratchpadExecutor;
    let pat = FixedPattern::new(Pattern::mats_plus(), size, 1);
    execute(pat, ex).expect("failed to run MATS+ pattern");
}

#[test]
#[ignore = "requires test chip"]
fn march_cm_bebe_scratchpad() {
    let size = STAC_SCRATCHPAD_SIZE;
    let ex = BebeScratchpadExecutor;
    let pat = FixedPattern::new(Pattern::march_cm(), size, 1);
    execute(pat, ex).expect("failed to run March C- pattern");
}

#[test]
#[ignore = "requires test chip"]
fn rand_bebe_scratchpad() {
    let size = STAC_SCRATCHPAD_SIZE;
    let ex = BebeScratchpadExecutor;
    let pat = FixedPattern::new(Pattern::rand(size.depth as u64 * 8), size, 151);
    execute(pat, ex).expect("failed to run random pattern");
}
