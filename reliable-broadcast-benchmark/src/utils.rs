use async_std::task::sleep;
use std::time::{Duration, Instant};

pub async fn sleep_until(start_time: Instant) {
    let remaining = start_time.saturating_duration_since(Instant::now());

    if remaining > Duration::ZERO {
        sleep(remaining).await;
    }
}
