#[allow(clippy::module_inception)]
pub mod stage;
pub mod stage_service;
mod stage_worker;
pub mod tasks;
pub use tasks::generate_task::GenerateTask;

pub fn safe_read(path: &str) -> Vec<u8> {
    tracing::debug!("read {}", path);
    std::fs::read(path).unwrap_or_else(|_e| {
        // tracing::warn!("read: {}, {:?}", path, e);
        vec![]
    })
}
