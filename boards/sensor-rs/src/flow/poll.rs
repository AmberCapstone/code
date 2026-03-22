use embassy_time::{Duration, Timer};

pub async fn until(mut condition: impl FnMut() -> bool, interval: Duration) {
    while !condition() {
        Timer::after(interval).await;
    }
}
