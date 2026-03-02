use criterion::{criterion_group, criterion_main, Criterion};
use zero_core::runtime::{DataPlane, StepSpec};

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

fn bench_data_plane_idempotency(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("data_plane_idempotency", |b| {
        b.iter(|| {
            rt.block_on(async {
                let dp = DataPlane::new_for_test();
                let step = StepSpec {
                    task_id: "t1".to_string(),
                    step_id: "s1".to_string(),
                    op: "agent.execute".to_string(),
                    idempotency_key: "k1".to_string(),
                };
                let _ = dp.execute_step(step.clone()).await.unwrap();
                let _ = dp.execute_step(step).await.unwrap();
            });
        });
    });
}

criterion_group!(benches, bench_vec_push, bench_data_plane_idempotency);
criterion_main!(benches);
