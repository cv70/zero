// Lightweight perf monitoring stub
pub fn log_perf_event(name: &str, value: f64) {
    // In a real system, replace with structured metrics emission
    println!("[perf] {} = {}", name, value);
}
