#![cfg(feature = "nightly")]
#![feature(test)]

use test::{bench, black_box};

extern crate test;

use cfg_examples::c::grammar;

#[bench]
fn bench_clone(bencher: &mut bench::Bencher) {
    let my_cfg = grammar();
    bencher.iter(|| {
        let copied = black_box(&my_cfg).clone();
        black_box(&copied);
    });
}
