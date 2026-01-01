use tokio::sync::broadcast;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusMessage {
    pub job_name: String,
    pub status: String,
    pub timestamp: i64,
}

pub type TaskStatusSender = broadcast::Sender<TaskStatusMessage>;

#[allow(dead_code)]
pub type TaskStatusReceiver = broadcast::Receiver<TaskStatusMessage>;

pub fn create_broadcast_channel() -> TaskStatusSender {
    let (tx, _rx) = broadcast::channel(100);
    tx
}

pub fn broadcast_task_status(
    sender: &TaskStatusSender,
    job_name: String,
    status: String,
) {
    let msg = TaskStatusMessage {
        job_name,
        status,
        timestamp: chrono::Utc::now().timestamp_millis(),
    };
    
    let job_name = msg.job_name.clone();
    let status = msg.status.clone();
    let _ = sender.send(msg);
    tracing::debug!("广播任务状态: {} -> {}", job_name, status);
}

