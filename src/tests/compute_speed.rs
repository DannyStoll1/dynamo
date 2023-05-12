use crate::profiles::QuadRatPer3;

#[test]
fn compute_per3() {
    let per3 = QuadRatPer3::new_default(1024, 2048);
    per3.compute();
}
