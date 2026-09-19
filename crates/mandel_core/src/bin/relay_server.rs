//! HAAVK Relink 公网中继服务端
//!
//! 用法: relay-server [--port 42070]
//!
//! 功能：
//! - TCP 监听，接受多个 HAAVK 节点连接
//! - 转发节点间的 Hello / AppMessage
//! - 跨网段节点通过此中继互联（替代局域网 UDP 广播）

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

type PeerMap = Arc<Mutex<HashMap<String, TcpStream>>>;

fn main() {
    let port: u16 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(42070);

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).expect(&format!("无法绑定 {addr}"));
    let peers: PeerMap = Arc::new(Mutex::new(HashMap::new()));

    println!("HAAVK Relink 中继服务端已启动: {addr}");
    println!("等待 HAAVK 节点连接...");

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("连接接受失败: {e}");
                continue;
            }
        };
        let peers = peers.clone();
        thread::spawn(move || {
            handle_client(stream, peers);
        });
    }
}

fn handle_client(mut stream: TcpStream, peers: PeerMap) {
    let mut buf = [0u8; 2048];
    let peer_addr = stream.peer_addr().unwrap_or_else(|_| "?".parse().unwrap());
    let mut node_id = format!("node-{peer_addr}");

    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let data = &buf[..n];
                // 尝试解析为 RelinkMessage
                if let Ok(msg) = serde_json::from_slice::<mandel_core::relink::RelinkMessage>(data) {
                    match &msg {
                        mandel_core::relink::RelinkMessage::Hello { node_id: nid, .. } => {
                            node_id = nid.clone();
                            println!("节点上线: {node_id} ({peer_addr})");
                            peers.lock().unwrap().insert(node_id.clone(), stream.try_clone().unwrap());
                        }
                        mandel_core::relink::RelinkMessage::AppMessage { from, to, kind, .. } => {
                            println!("转发: {from} → {to} ({kind})");
                            // 转发给目标节点
                            if let Some(target) = peers.lock().unwrap().get(to) {
                                let mut target = target;
                                let _ = target.write_all(data);
                            }
                        }
                    }
                }
            }
            Err(_) => break,
        }
    }

    peers.lock().unwrap().remove(&node_id);
    println!("节点离线: {node_id} ({peer_addr})");
}
