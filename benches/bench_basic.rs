use criterion::{criterion_group, criterion_main, Criterion};

fn bench_vec_push(c: &mut Criterion) {
    c.bench_function("vec_push_1024", |b| {
        b.iter(|| {
            let mut v = Vec::with_capacity(1024);
            for i in 0..1024 {
                v.push(i);
            }
        });
    });
}

criterion_group!(benches, bench_vec_push);
criterion_main!(benches);
