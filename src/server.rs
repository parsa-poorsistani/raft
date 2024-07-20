use crate::{raft::RaftRPCClient, RaftNode};
use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc, time::Duration};
use tarpc::serde_transport::tcp;
use tarpc::{client, context};
use tokio::{sync::Mutex, time::timeout};

pub async fn call<C, Args, Reply>(
    client: &C,
    method: impl Fn(
        C,
        context::Context,
        Args,
    ) -> Pin<Box<dyn Future<Output = Result<Reply, tarpc::client::RpcError>> + Send>>,
    args: Args,
) -> Result<Reply, String>
where
    C: Clone,
{
    let ctx = context::current();
    match timeout(Duration::from_secs(1), method(client.clone(), ctx, args)).await {
        Ok(Ok(reply)) => Ok(reply),
        Ok(Err(e)) => Err(format!("RPC error: {}", e)),
        Err(_) => Err("RPC timeout".to_string()),
    }
}

pub struct RaftServer {
    node: Arc<Mutex<RaftNode>>,
    clients: HashMap<String, RaftRPCClient>,
}

impl RaftServer {
    pub async fn new(node: RaftNode) -> Result<Self> {
        let mut clients = HashMap::new();
        for peer in &node.peers {
            let peer_addr: SocketAddr = peer.parse().context("Failed to parse the peer address")?;

            let transport = tarpc::serde_transport::tcp::connect(
                peer_addr,
                tarpc::serde_transport::tcp::Json::,
            )
            .await
            .with_context(|| format!("Failed to establish TCP to peer {}", peer))?;
            let client = RaftRPCClient::new(client::Config::default(), transport).spawn();
            clients.insert(peer, client);
        }
        Ok(RaftServer {
            node: Arc::new(Mutex::new(node)),
            clients,
        })
    }
}
