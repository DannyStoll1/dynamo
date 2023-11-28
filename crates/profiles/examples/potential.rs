#![allow(dead_code)]
#![allow(clippy::unwrap_used)]
use dynamo_common::prelude::*;
use dynamo_core::prelude::*;
use dynamo_profiles::Mandelbrot;

fn escape()
{
    let c = Cplx::new(-0.75, 1e-7);

    let mandelbrot = Mandelbrot::default().with_max_iter(4_294_967_296);
    let mut orbit = orbit::Potential::new(&mandelbrot).init(c);
    let (green, d_green) = orbit.run_until_complete().unwrap();

    println!("               c = {c}");
    println!("          -log G = {green}");
    println!("      ∇ (-log G) = {}", -d_green);
    println!("iters ≈ -log2(G) = {}", green / std::f64::consts::LN_2);
}

fn bottcher()
{
    const EPS: Real = 1e-6;

    let c = Cplx::new(-1.0, -0.0);
    let z = Cplx::new(0.3, 0.1);

    let mandelbrot = Mandelbrot::default().with_max_iter(65536);
    let julia = JuliaSet::from(mandelbrot).with_param(c);
    let mut orbit = orbit::Potential::new(&julia).init(z);
    let (green, d_green) = orbit.run_until_complete().unwrap();

    orbit.reset(z + EPS);
    let (green_d_re, _) = orbit.run_until_complete().unwrap();

    orbit.reset(z + Cplx::new(0.0, EPS));
    let (green_d_im, _) = orbit.run_until_complete().unwrap();

    let deriv_est_re = (green_d_re - green) / EPS;
    let deriv_est_im = (green_d_im - green) / EPS;
    println!("ϕ = {green}");
    println!("∇ϕ = {d_green}");
    println!("∇ϕ est = {}", Cplx::new(deriv_est_re, deriv_est_im));
}

fn koenigs()
{
    const EPS: Real = 1e-6;

    let c = Cplx::new(-0.4, -0.3);
    let z = Cplx::new(0.2, 0.1);

    let mandelbrot = Mandelbrot::default().with_max_iter(65536);
    let julia = JuliaSet::from(mandelbrot).with_param(c);
    let mut orbit = orbit::Potential::new(&julia).init(z);
    let (phi, d_phi) = orbit.run_until_complete().unwrap();

    orbit.reset(z + EPS);
    let (phi_d_re, _) = orbit.run_until_complete().unwrap();

    orbit.reset(z + Cplx::new(0.0, EPS));
    let (phi_d_im, _) = orbit.run_until_complete().unwrap();

    let deriv_est_re = (phi_d_re - phi) / EPS;
    let deriv_est_im = (phi_d_im - phi) / EPS;
    println!("log log |φ| = {phi}");
    println!("∇ log log |φ| = {d_phi}");
    println!(
        "∇ log log |φ| est = {}",
        Cplx::new(deriv_est_re, deriv_est_im)
    );
}

fn hyp_component_interior()
{
    const EPS: Real = 1e-8;
    let c = Cplx::new(-0.5, 0.2);

    let mandelbrot = Mandelbrot::default().with_max_iter(4_294_967_296);
    let mut orbit = orbit::Potential::new(&mandelbrot).init(c);
    let (mult, d_mult) = orbit.run_until_complete().unwrap();

    orbit.reset(c + EPS);
    let (mult_d_re, _) = orbit.run_until_complete().unwrap();

    orbit.reset(c + Cplx::new(0.0, EPS));
    let (mult_d_im, _) = orbit.run_until_complete().unwrap();

    let deriv_est_re = (mult_d_re - mult) / EPS;
    let deriv_est_im = (mult_d_im - mult) / EPS;
    let d_mult_est = Cplx::new(deriv_est_re, deriv_est_im);

    let mu_true = 1. - (1. - 4. * c).sqrt();
    let mu_true_d = 2. / (1. - 4. * c).sqrt();

    println!("c        = {c}");
    println!("μ        = {mult}");
    println!("μ true   = {}", mu_true.norm_sqr());
    println!("∇ μ      = {d_mult}");
    println!("∇ μ est  = {d_mult_est}");
    println!("∇ μ true = {}", 2. * mu_true_d.conj() * mu_true);
}

fn main()
{
    escape();
}
