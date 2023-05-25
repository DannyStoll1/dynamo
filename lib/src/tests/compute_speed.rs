use crate::profiles::QuadRatPer3;

#[test]
fn compute_per3()
{
    let per3 = QuadRatPer3::default().with_res_y(1024).with_max_iter(2048);
    per3.compute();
}
