//! PostgreSQL 定向慢操作计时，不记录语句参数或错误正文

use std::{
    future::Future,
    time::{Duration, Instant},
};

const SLOW_OPERATION_THRESHOLD: Duration = Duration::from_millis(100);

struct OperationTrace {
    operation: &'static str,
    phase: &'static str,
    started: Instant,
    outcome: &'static str,
}

impl Drop for OperationTrace {
    fn drop(&mut self) {
        let elapsed = self.started.elapsed();
        if elapsed >= SLOW_OPERATION_THRESHOLD || self.outcome == "error" {
            tracing::warn!(
                operation = self.operation,
                phase = self.phase,
                outcome = self.outcome,
                elapsed_ms = elapsed.as_millis() as u64,
                threshold_ms = SLOW_OPERATION_THRESHOLD.as_millis() as u64,
                "PostgreSQL operation trace"
            );
        }
    }
}

pub(super) async fn trace_operation<T, E>(
    operation: &'static str,
    phase: &'static str,
    future: impl Future<Output = Result<T, E>>,
) -> Result<T, E> {
    let mut trace = OperationTrace {
        operation,
        phase,
        started: Instant::now(),
        outcome: "cancelled",
    };
    let result = future.await;
    trace.outcome = if result.is_ok() { "success" } else { "error" };
    result
}
