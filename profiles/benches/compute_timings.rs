#![feature(test)]

extern crate test;
use test::Bencher;

use dynamo_common::prelude::*;
use dynamo_core::prelude::*;
use dynamo_profiles::*;

#[bench]
fn compute_biquadratic(b: &mut Bencher)
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
fn compute_per2(b: &mut Bencher)
{
    b.iter(|| {
        let plane = QuadRatPer2::default().with_res_y(768).with_max_iter(2048);
        plane.compute();
    });
}

#[bench]
fn compute_per2_julia(b: &mut Bencher)
{
    b.iter(|| {
        let plane = QuadRatPer2::default().with_res_y(1024).with_max_iter(2048);
        let julia = JuliaSet::from(plane);
        let mut iter_plane = julia.compute();
        for _ in 0..9
        {
            julia.compute_into(&mut iter_plane);
        }
    });
}

#[bench]
fn compute_per3(b: &mut Bencher)
{
    b.iter(|| {
        let plane = QuadRatPer3::default().with_res_y(768).with_max_iter(2048);
        plane.compute();
    });
}

#[bench]
fn compute_per4(b: &mut Bencher)
{
    b.iter(|| {
        let plane = QuadRatPer4::default().with_res_y(768).with_max_iter(2048);
        plane.compute();
    });
}

#[bench]
fn compute_per5(b: &mut Bencher)
{
    b.iter(|| {
        let plane = QuadRatPer5::default().with_res_y(768).with_max_iter(2048);
        plane.compute();
    });
}

#[bench]
fn compute_mandelbrot(b: &mut Bencher)
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
fn compute_preper21(b: &mut Bencher)
{
    b.iter(|| {
        let plane = QuadRatPreper21::default()
            .with_res_y(768)
            .with_max_iter(2048);
        plane.compute();
    });
}

#[bench]
fn compute_symmetry_locus(b: &mut Bencher)
{
    b.iter(|| {
        let plane = QuadRatSymmetryLocus::default()
            .with_res_y(512)
            .with_max_iter(2048);
        plane.compute();
    });
}

#[bench]
fn compute_rulkov(b: &mut Bencher)
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
