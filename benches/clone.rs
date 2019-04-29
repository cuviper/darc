#[macro_use]
extern crate criterion;

use criterion::Criterion;
use darc::{Arc, Rc};
use std::rc::Rc as StdRc;
use std::sync::Arc as StdArc;

fn data() -> Vec<i32> {
    (0..1_000_000).collect()
}

fn bench_rc_clone(c: &mut Criterion) {
    let rc = Rc::new(data());
    c.bench_function("darc::Rc clone", move |b| b.iter(|| rc.clone()));

    let arc = Arc::new(data());
    let rc = Rc::from_arc(arc);
    c.bench_function("darc::Rc shared clone", move |b| b.iter(|| rc.clone()));

    let rc = StdRc::new(data());
    c.bench_function("std::rc::Rc clone", move |b| b.iter(|| rc.clone()));
}

fn bench_arc_clone(c: &mut Criterion) {
    let arc = Arc::new(data());
    c.bench_function("darc::Arc clone", move |b| b.iter(|| arc.clone()));

    let arc = StdArc::new(data());
    c.bench_function("std::sync::Arc clone", move |b| b.iter(|| arc.clone()));
}

criterion_group!(benches, bench_rc_clone, bench_arc_clone);
criterion_main!(benches);
