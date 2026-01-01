use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};

use crate::app::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.ws_sender.subscribe();
    
    // 发送初始连接成功消息
    let init_msg = serde_json::json!({
        "type": "connected",
        "message": "WebSocket 连接已建立"
    });
    
    if sender.send(axum::extract::ws::Message::Text(init_msg.to_string())).await.is_err() {
        return;
    }
    
    // 处理来自客户端的消息（心跳等）
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if matches!(msg, axum::extract::ws::Message::Close(_)) {
                break;
            }
        }
    });
    
    // 处理广播消息并发送给客户端
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            
            if sender
                .send(axum::extract::ws::Message::Text(json))
                .await
                .is_err()
            {
                break;
            }
        }
    });
    
    // 等待任一任务完成
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
    
    tracing::info!("WebSocket 连接已关闭");
}

