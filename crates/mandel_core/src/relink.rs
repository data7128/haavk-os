//! Relink 同频 IPC 总线（Stage 3 完整化）。
//!
//! 协议：UDP 广播，端口 42069
//! - 启动时每秒广播 `Hello{node_id, version, product}` 包
//! - 收到 Hello 包 → 记录/更新对端节点（节点发现）
//! - 节点列表用于「同频节点」面板展示
//!
//! 曼德尔核心不直接操作网卡，仅在用户态收发 UDP 数据报；
//! 广播/组播的具体实现委托宿主内核网络栈。

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use log::{info, warn};
use serde::{Deserialize, Serialize};

use crate::config::MandelConfig;

/// 一个远程 HAAVK 节点
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 对端节点快照
#[derive(Debug, Clone)]
pub struct PeerNode {
    pub id: String,
    pub addr: SocketAddr,
    pub product: String,
    pub version: String,
    pub last_seen: Instant,
}

/// 广播消息（UDP 数据报载荷）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelinkMessage {
    Hello {
        node_id: String,
        product: String,
        version: String,
    },
    /// 应用间消息（Stage 3 预留）
    AppMessage {
        from: String,
        to: String,
        kind: String,
        payload: String,
    },
}

/// Relink 同频总线
pub struct RelinkBus {
    pub enabled: bool,
    pub node_discovery: bool,
    pub discovery_port: u16,
    pub max_connections: u32,
    running: bool,
    peers: Arc<Mutex<HashMap<String, PeerNode>>>,
    stop_flag: Arc<AtomicBool>,
    socket: Option<UdpSocket>,
}

impl RelinkBus {
    pub fn new(config: &MandelConfig) -> Self {
        Self {
            enabled: config.relink_bus.enable,
            node_discovery: config.relink_bus.node_discovery,
            discovery_port: config.relink_bus.discovery_port,
            max_connections: config.relink_bus.max_connections,
            peers: Arc::new(Mutex::new(HashMap::new())),
            stop_flag: Arc::new(AtomicBool::new(false)),
            socket: None,
            running: false,
        }
    }

    /// 启动总线：绑定 UDP socket，启动广播线程 + 监听线程
    pub fn start(&mut self, local_node_id: &str, product: &str, version: &str) {
        if self.running {
            return;
        }
        self.running = true;

        // 绑定监听端口
        let bind_addr: SocketAddr = format!("0.0.0.0:{}", self.discovery_port)
            .parse()
            .unwrap_or_else(|_| ([0, 0, 0, 0], self.discovery_port).into());

        match UdpSocket::bind(bind_addr) {
            Ok(sock) => {
                let _ = sock.set_broadcast(true);
                let _ = sock.set_read_timeout(Some(Duration::from_millis(300)));
                info!(
                    "Relink 同频总线已绑定 {}（节点发现={}）",
                    bind_addr, self.node_discovery
                );
                self.socket = Some(sock);
            }
            Err(e) => {
                warn!("Relink UDP 绑定失败（{}），同频发现降级为离线模式", e);
                self.running = false;
                return;
            }
        }

        // 广播线程：每秒发 Hello
        if self.node_discovery {
            let snd = self.socket.as_ref().unwrap().try_clone().unwrap();
            let peers = self.peers.clone();
            let stop = self.stop_flag.clone();
            let node_id = local_node_id.to_string();
            let product = product.to_string();
            let version = version.to_string();
            let port = self.discovery_port;
            std::thread::spawn(move || {
                let bc = SocketAddr::from((Ipv4Addr::BROADCAST, port));
                let msg = RelinkMessage::Hello { node_id, product, version };
                loop {
                    if stop.load(Ordering::Relaxed) {
                        break;
                    }
                    if let Ok(bytes) = serde_json::to_vec(&msg) {
                        let _ = snd.send_to(&bytes, bc);
                    }
                    // 清理超时节点（10 秒未心跳）
                    peers.lock().unwrap().retain(|_, p| p.last_seen.elapsed() < Duration::from_secs(10));
                    std::thread::sleep(Duration::from_secs(1));
                }
            });
        }

        // 监听线程：收 Hello / AppMessage
        let rcv = self.socket.as_ref().unwrap().try_clone().unwrap();
        let peers = self.peers.clone();
        let stop = self.stop_flag.clone();
        std::thread::spawn(move || {
            let mut buf = [0u8; 2048];
            loop {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                match rcv.recv_from(&mut buf) {
                    Ok((len, addr)) => {
                        let data = &buf[..len];
                        if let Ok(msg) = serde_json::from_slice::<RelinkMessage>(data) {
                            match msg {
                                RelinkMessage::Hello { node_id, product, version } => {
                                    let mut map = peers.lock().unwrap();
                                    map.insert(
                                        node_id.clone(),
                                        PeerNode {
                                            id: node_id,
                                            addr,
                                            product,
                                            version,
                                            last_seen: Instant::now(),
                                        },
                                    );
                                }
                                RelinkMessage::AppMessage { from, kind, .. } => {
                                    info!("Relink 应用消息：{} → {}", from, kind);
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // 读超时，循环继续检查 stop
                    }
                }
            }
        });

        info!(
            "Relink 同频总线就绪：节点发现=true 端口={} 最大连接={}",
            self.discovery_port, self.max_connections
        );
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        info!("Relink 同频总线已断开");
    }

    /// 当前在线对端节点快照
    pub fn peers(&self) -> Vec<PeerNode> {
        self.peers.lock().unwrap().values().cloned().collect()
    }

    pub fn peer_count(&self) -> usize {
        self.peers.lock().unwrap().len()
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}

/// 公网中继客户端（Stage 4 骨架）：跨网段节点通过中继服务器互联
///
/// 协议：TCP 长连接到中继服务器（默认 haavk://relay ）
/// - 连接后发送 `Hello{node_id}` 注册
/// - 中继服务器转发其他节点的 AppMessage
/// - 网络断开自动重连（指数退避）
pub struct RelayClient {
    pub server_addr: String,
    pub enabled: bool,
    connected: bool,
}

impl RelayClient {
    pub fn new(server_addr: &str, enabled: bool) -> Self {
        Self { server_addr: server_addr.into(), enabled, connected: false }
    }

    /// 尝试连接中继服务器（骨架：仅建立 TCP 连接，不做完整握手）
    pub fn connect(&mut self) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }
        match std::net::TcpStream::connect(&self.server_addr) {
            Ok(_stream) => {
                self.connected = true;
                info!("公网中继已连接：{}", self.server_addr);
                Ok(())
            }
            Err(e) => {
                self.connected = false;
                warn!("公网中继连接失败（{}），将使用局域网广播模式", e);
                Err(format!("中继连接失败: {e}"))
            }
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relink_message_serde() {
        let m = RelinkMessage::Hello {
            node_id: "HAAVK-NODE-00001".into(),
            product: "HAAVK全域算力终端".into(),
            version: "0.1.0-mvp".into(),
        };
        let bytes = serde_json::to_vec(&m).unwrap();
        let back: RelinkMessage = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn bus_creation() {
        let cfg = MandelConfig::default();
        let bus = RelinkBus::new(&cfg);
        assert!(bus.enabled);
        assert_eq!(bus.discovery_port, 42069);
        assert_eq!(bus.peer_count(), 0);
    }
}
