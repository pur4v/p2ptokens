//! libp2p node: TCP + Noise + Yamux carrying the completion protocol, with the
//! full NAT-traversal stack so peers behind home routers can still connect
//! (design Q3/Q12):
//!   - identify   — exchange observed addresses
//!   - autonat    — detect whether we're publicly reachable
//!   - upnp       — try to auto-open a router port (like a torrent client)
//!   - relay      — client: reserve a slot on a public relay; server (--relay):
//!                  be that public rendezvous for others
//!   - dcutr      — hole-punch a direct connection through the relay
//!
//! This mirrors how modern torrent clients connect: UPnP + a connectable
//! (relay) peer + hole-punching, with the relay as fallback.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{anyhow, Result};
use futures::StreamExt;
use libp2p::multiaddr::Protocol;
use libp2p::swarm::behaviour::toggle::Toggle;
use libp2p::swarm::{dial_opts::DialOpts, NetworkBehaviour, SwarmEvent};
use libp2p::{autonat, dcutr, identify, ping, relay, upnp};
use libp2p::{Multiaddr, PeerId, StreamProtocol};
use libp2p_stream as stream;
use tokio::sync::{mpsc, oneshot};

use p2ptokens_shared::crypto;
use p2ptokens_shared::protocol::COMPLETION_PROTOCOL;

#[derive(NetworkBehaviour)]
struct Behaviour {
    stream: stream::Behaviour,
    identify: identify::Behaviour,
    ping: ping::Behaviour,
    relay_client: Toggle<relay::client::Behaviour>,
    dcutr: Toggle<dcutr::Behaviour>,
    autonat: Toggle<autonat::Behaviour>,
    upnp: Toggle<upnp::tokio::Behaviour>,
    relay_server: Toggle<relay::Behaviour>,
}

enum Command {
    Connect {
        peer: PeerId,
        addrs: Vec<Multiaddr>,
        reply: oneshot::Sender<Result<(), String>>,
    },
}

#[derive(Clone)]
pub struct NodeHandle {
    control: stream::Control,
    cmd_tx: mpsc::Sender<Command>,
    listen_addrs: Arc<Mutex<Vec<Multiaddr>>>,
    local_peer: PeerId,
}

impl NodeHandle {
    pub fn control(&self) -> stream::Control {
        self.control.clone()
    }

    pub fn listen_addrs(&self) -> Vec<Multiaddr> {
        self.listen_addrs.lock().unwrap().clone()
    }

    /// Dial a peer (by id, with candidate addresses — direct or /p2p-circuit)
    /// and wait for a connection.
    pub async fn connect(&self, peer: PeerId, addrs: Vec<Multiaddr>) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(Command::Connect {
                peer,
                addrs,
                reply: tx,
            })
            .await
            .map_err(|_| anyhow!("node task gone"))?;
        rx.await
            .map_err(|_| anyhow!("no dial reply"))?
            .map_err(|e| anyhow!(e))
    }
}

pub struct NodeConfig {
    pub listen: Multiaddr,
    /// run as a public relay server (rendezvous for NAT'd peers)
    pub relay: bool,
    /// address of a relay to reserve a slot on (for reachability behind NAT)
    pub relay_addr: Option<Multiaddr>,
}

pub async fn start(
    keypair: crypto::Identity,
    cfg: NodeConfig,
) -> Result<(NodeHandle, stream::IncomingStreams)> {
    let is_relay = cfg.relay;
    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_relay_client(libp2p::noise::Config::new, libp2p::yamux::Config::default)?
        .with_behaviour(|key, relay_client| {
            let pid = key.public().to_peer_id();
            Behaviour {
                stream: stream::Behaviour::new(),
                identify: identify::Behaviour::new(identify::Config::new(
                    "/p2ptokens/id/1.0.0".into(),
                    key.public(),
                )),
                ping: ping::Behaviour::new(ping::Config::new()),
                relay_client: Toggle::from(Some(relay_client)),
                dcutr: Toggle::from(Some(dcutr::Behaviour::new(pid))),
                autonat: Toggle::from(Some(autonat::Behaviour::new(
                    pid,
                    autonat::Config::default(),
                ))),
                upnp: Toggle::from(Some(upnp::tokio::Behaviour::default())),
                relay_server: Toggle::from(if is_relay {
                    Some(relay::Behaviour::new(pid, relay::Config::default()))
                } else {
                    None
                }),
            }
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(120)))
        .build();

    let local_peer = *swarm.local_peer_id();
    let mut control = swarm.behaviour().stream.new_control();
    let incoming = control.accept(StreamProtocol::new(COMPLETION_PROTOCOL))?;

    swarm.listen_on(cfg.listen)?;

    // If we were given a relay, dial it now. We reserve the /p2p-circuit slot
    // only once the connection is established (reserving too early races the
    // connection and the listener closes). DCUtR then upgrades to direct.
    let relay_target: Option<(PeerId, Multiaddr)> = cfg.relay_addr.as_ref().and_then(|a| {
        let peer = a.iter().find_map(|p| match p {
            Protocol::P2p(id) => Some(id),
            _ => None,
        })?;
        Some((peer, a.clone().with(Protocol::P2pCircuit)))
    });
    if let Some(addr) = cfg.relay_addr.clone() {
        if let Err(e) = swarm.dial(addr) {
            tracing::warn!("dial relay failed: {e}");
        }
    }

    let (cmd_tx, mut cmd_rx) = mpsc::channel::<Command>(32);
    let listen_addrs = Arc::new(Mutex::new(Vec::new()));
    let handle = NodeHandle {
        control: control.clone(),
        cmd_tx,
        listen_addrs: listen_addrs.clone(),
        local_peer,
    };

    let mut pending: HashMap<PeerId, Vec<oneshot::Sender<Result<(), String>>>> = HashMap::new();
    let mut reserved = false;

    tokio::spawn(async move {
        loop {
            tokio::select! {
                cmd = cmd_rx.recv() => match cmd {
                    Some(Command::Connect { peer, addrs, reply }) => {
                        if swarm.is_connected(&peer) {
                            let _ = reply.send(Ok(()));
                            continue;
                        }
                        let opts = DialOpts::peer_id(peer).addresses(addrs).build();
                        match swarm.dial(opts) {
                            Ok(()) => pending.entry(peer).or_default().push(reply),
                            Err(e) => { let _ = reply.send(Err(e.to_string())); }
                        }
                    }
                    None => break,
                },
                event = swarm.select_next_some() => match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        tracing::info!("listening on {address}");
                        // a relay is meant to be public: advertise its listen
                        // address as external so reservations include it (else
                        // clients can't form their relayed address).
                        if is_relay {
                            swarm.add_external_address(address.clone());
                        }
                        listen_addrs.lock().unwrap().push(address);
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        if let Some(list) = pending.remove(&peer_id) {
                            for r in list { let _ = r.send(Ok(())); }
                        }
                        // once connected to our relay, reserve a circuit slot
                        if let Some((rp, circuit)) = &relay_target {
                            if peer_id == *rp && !reserved {
                                reserved = true;
                                match swarm.listen_on(circuit.clone()) {
                                    Ok(_) => tracing::info!("reserving relay slot: {circuit}"),
                                    Err(e) => tracing::warn!("relay reserve failed: {e}"),
                                }
                            }
                        }
                    }
                    SwarmEvent::OutgoingConnectionError { peer_id: Some(peer_id), error, .. } => {
                        if let Some(list) = pending.remove(&peer_id) {
                            for r in list { let _ = r.send(Err(error.to_string())); }
                        }
                    }
                    SwarmEvent::ExternalAddrConfirmed { address } => {
                        tracing::info!("external address confirmed: {address}");
                    }
                    SwarmEvent::Behaviour(ev) => match ev {
                        BehaviourEvent::Dcutr(e) => tracing::info!("dcutr: {e:?}"),
                        BehaviourEvent::Autonat(e) => tracing::debug!("autonat: {e:?}"),
                        BehaviourEvent::RelayServer(e) => tracing::debug!("relay-server: {e:?}"),
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    });

    Ok((handle, incoming))
}
