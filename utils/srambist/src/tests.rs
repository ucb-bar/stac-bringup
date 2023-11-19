use crate::executor::{execute, IdealExecutor};
use crate::pattern::{FixedPattern, Pattern, SramSize};

#[test]
fn mats_plus_ideal_executor() {
    let size = SramSize::new(32, 256, 4);
    let ex = IdealExecutor::new(size);
    let pat = FixedPattern::new(Pattern::mats_plus(), size, 1);
    execute(pat, ex).expect("MATS+ pattern should execute correctly with an ideal executor");
}