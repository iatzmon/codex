use codex_core::hooks::HookRegistry;
use codex_core::hooks::executor::{HookExecutor, PreToolUsePayload};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn bench_pretool_allow(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().expect("create tokio runtime");
    let executor = HookExecutor::with_registry(HookRegistry::default());

    c.bench_function("hook_executor_pretool_allow", |b| {
        b.iter(|| {
            let payload = PreToolUsePayload {
                tool_name: "shell".to_string(),
                command: "echo benchmark".to_string(),
            };
            runtime
                .block_on(executor.evaluate_pre_tool_use(black_box(&payload)))
                .expect("pretool result");
        });
    });
}

criterion_group!(benches, bench_pretool_allow);
criterion_main!(benches);
