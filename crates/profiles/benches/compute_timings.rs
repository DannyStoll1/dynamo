#![feature(test)]

extern crate test;
use test::Bencher;

use dynamo_common::prelude::*;
use dynamo_core::prelude::*;
use dynamo_profiles::*;

#[bench]
fn biquadratic(b: &mut Bencher)
{
    b.iter(|| {
        let plane = Biquadratic::default()
            .with_res_y(768)
            .with_max_iter(2048)
            .with_param((-0.3).into());
        plane.compute();
    });
}

#[bench]
fn per2(b: &mut Bencher)
{
    b.iter(|| {
        let plane = QuadRatPer2::default().with_res_y(768).with_max_iter(2048);
        plane.compute();
    });
}

#[bench]
fn per2_julia(b: &mut Bencher)
{
    b.iter(|| {
        let plane = QuadRatPer2::default().with_res_y(1024).with_max_iter(2048);
        let julia = JuliaSet::from(plane);
        let mut iter_plane = julia.compute();
        for _ in 0..9 {
            julia.compute_into(&mut iter_plane);
        }
    });
}

#[bench]
fn quadrat_per3(b: &mut Bencher)
{
    b.iter(|| {
        let plane = QuadRatPer3::default().with_res_y(768).with_max_iter(2048);
        plane.compute();
    });
}

#[bench]
fn per4(b: &mut Bencher)
{
    b.iter(|| {
        let plane = QuadRatPer4::default().with_res_y(768).with_max_iter(2048);
        plane.compute();
    });
}

#[bench]
fn per5(b: &mut Bencher)
{
    b.iter(|| {
        let plane = QuadRatPer5::default().with_res_y(768).with_max_iter(2048);
        plane.compute();
    });
}

#[bench]
fn mandelbrot(b: &mut Bencher)
{
    b.iter(|| {
        let plane = Mandelbrot::default().with_res_y(768).with_max_iter(2048);
        plane.compute();
    });
}

#[bench]
fn quadratic_julia(b: &mut Bencher)
{
    b.iter(|| {
        let mut plane = JuliaSet::from(Mandelbrot::default().with_res_y(768).with_max_iter(2048));
        plane.set_param(0.25.into());
        plane.compute();
    });
}

#[bench]
fn preper21(b: &mut Bencher)
{
    b.iter(|| {
        let plane = QuadRatPreper21::default()
            .with_res_y(768)
            .with_max_iter(2048);
        plane.compute();
    });
}

#[bench]
fn symmetry_locus(b: &mut Bencher)
{
    b.iter(|| {
        let plane = QuadRatSymmetryLocus::default()
            .with_res_y(512)
            .with_max_iter(2048);
        plane.compute();
    });
}

#[bench]
fn rulkov(b: &mut Bencher)
{
    b.iter(|| {
        let mut plane = Rulkov::default().with_res_y(768).with_max_iter(2048);
        let bounds = Bounds {
            min_x: -9.859_092_283_464_022,
            max_x: 11.712_293_384_188_932,
            min_y: -11.185_367_984_659_074,
            max_y: 10.011_947_411_300_476,
        };

        plane.point_grid_mut().change_bounds(bounds);
        plane.compute();
    });
}

#[bench]
fn cubic_per3(b: &mut Bencher)
{
    b.iter(|| {
        let plane = CubicPer3_0::default().with_res_y(1024).with_max_iter(2048);
        plane.compute();
    });
}

#[bench]
fn exp(b: &mut Bencher)
{
    use test::black_box;
    b.iter(|| {
        let mut r = 0.42_f64;
        for _ in 0..10_000 {
            let u = Cplx::new(r, r.ln()).exp();
            black_box(u);
            r += TAU % 1.0;
        }
    });
}

#[bench]
fn exp2(b: &mut Bencher)
{
    use test::black_box;
    b.iter(|| {
        let mut r = 0.42_f64;
        for _ in 0..10_000 {
            let u = Cplx::new(r, r.log2()).exp2();
            black_box(u);
            r += TAU % 1.0;
        }
    });
}

#[bench]
fn norm(b: &mut Bencher)
{
    use test::black_box;
    b.iter(|| {
        let mut r = 0.42_f64;
        for _ in 0..10_000 {
            let u = Cplx::new(r, 1.3).norm();
            black_box(u);
            r += TAU % 1.0;
        }
    });
}

#[bench]
fn norm_sqr(b: &mut Bencher)
{
    use test::black_box;
    b.iter(|| {
        let mut r = 0.42_f64;
        for _ in 0..10_000 {
            let u = Cplx::new(r, 1.3).norm_sqr();
            black_box(u);
            r += TAU % 1.0;
        }
    });
}
