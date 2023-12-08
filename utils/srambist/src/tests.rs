use crate::bebe::BebeScratchpadExecutor;
use crate::executor::{execute, IdealExecutor};
use crate::pattern::{FixedPattern, Pattern, SramSize};

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
    let size = SramSize::new(64, 512, 8);
    let ex = BebeScratchpadExecutor;
    let pat = FixedPattern::new(Pattern::mats_plus(), size, 1);
    execute(pat, ex).expect("failed to run MATS+ pattern");
}
