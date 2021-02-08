// Copyright 2020 Gnosis Ltd.
// SPDX-License-Identifier: Apache-2.0

extern crate ethcore_io as io;
extern crate ethcore_network as network;
extern crate ethcore_network_devp2p as devp2p;

use reth_scheduler::devp2p_adapter::{Devp2pAdapter, Devp2pInbound, PeerPenal};
use reth_scheduler::scheduler::{
    peer_organizer::PeerId,
    protocol::*,
};
use log::*;
use devp2p::NetworkService;
pub use io::TimerToken;
use network::{Error, NetworkConfiguration, NetworkContext, NetworkProtocolHandler};
use std::{collections::{HashMap, HashSet}, net::SocketAddr, str::FromStr, sync::Arc};

pub struct OeDevp2p {
    network: NetworkService,
}

impl OeDevp2p {
    // dummy for test, request config to start adapter
    // Currently hardcoded but expect to receive devp2p configuration here.
    pub fn new() -> Result<OeDevp2p, Error> {
        //lets start rust libs
        let mut config = NetworkConfiguration::default();
        config.client_version = "reth-scheduler-0.1.0".to_string();
        config.public_address = Some(SocketAddr::from_str("178.222.150.180:30303")?);
        //config.listen_address = Some(SocketAddr::from_str("localhost:30303")?);
        config.boot_nodes = vec!["enode://d860a01f9722d78051619d1e2351aba3f43f943f6f00718d1b9baa4101932a1f5011f16bb2b1bb35db20d6fe28fa0bf09636d26a87d31de9ec6203eeedb1f666@18.138.108.67:30303".to_string(),
    "enode://22a8232c3abc76a16ae9d6c3b164f98775fe226f0917b0ca871128a74a8e9630b458460865bab457221f1d448dd9791d24c4e5d88786180ac185df813a68d4de@3.209.45.79:30303".to_string(),
    "enode://ca6de62fce278f96aea6ec5a2daadb877e51651247cb96ee310a318def462913b653963c155a0ef6c7d50048bba6e6cea881130857413d9f50a621546b590758@34.255.23.113:30303".to_string(),
    "enode://5f7d0794c464b2fcd514d41e16e4b535a98ac792a71ca9667c7cef35595dc34c9a1b793c0622554cf87f34006942abb526af7d2e37d715ac32ed02170556cce2@51.161.101.207:30303".to_string()];
        config.nat_enabled = true;
        config.max_peers =100;
        config.min_peers =30;
        let network = NetworkService::new(config, None)?;
        network.start().expect(" TO BE OKAY"); //TODO test, create proper error handling
        Ok(OeDevp2p { network })
    }
}

/// Wrapper around OE handler, it receives adapter handler and binds OE Network handler to it.
pub struct HandleWrapper {
    protocol: ProtocolId,
    inbound_handler: Arc<dyn Devp2pInbound>,
}

impl HandleWrapper {
    pub fn new(protocol: ProtocolId, handler: Arc<dyn Devp2pInbound>) -> HandleWrapper {
        HandleWrapper {
            protocol,
            inbound_handler: handler,
        }
    }
}

impl NetworkProtocolHandler for HandleWrapper {
    /// Called when new network packet received.
    fn read(&self, _io: &dyn NetworkContext, peer: &PeerId, message_id: u8, data: &[u8]) {
        self.inbound_handler.receive_message(
            peer,
            self.protocol,
            message_id,
            data,
        );
    }

    /// Called when new peer is connected. Only called when peer supports the same protocol.
    fn connected(&self, io: &dyn NetworkContext, peer: &PeerId) {
        if self.protocol != ProtocolId::Eth {
            return;
        }
        let mut capability = HashMap::new();
        let session = io.session_info(*peer).unwrap();
        for capa in session.peer_capabilities.iter() {

            let protocol = match capa.protocol {
                //TODO add supported versions
                prot if prot == ProtocolId::Eth.to_protocol_type() => ProtocolId::Eth,
                prot if prot == ProtocolId::Parity.to_protocol_type() => ProtocolId::Parity,
                _ => continue,
            };
            capability.entry(protocol).or_insert(HashSet::new()).insert(capa.version);
        }
        self.inbound_handler.connected(peer, &capability);
    }

    /// Called when a previously connected peer disconnects.
    fn disconnected(&self, _io: &dyn NetworkContext, peer: &PeerId) {
        if self.protocol != ProtocolId::Eth {
            return;
        }
        self.inbound_handler.disconnected(peer);
    }
}

impl Devp2pAdapter for OeDevp2p {
    fn start(&self) {
        info!("Network already started");
    }

    fn stop(&self) {
        //TODO
    }

    fn register_handler(&self, handle: Arc<dyn Devp2pInbound>) {
        debug!("Register handle");
        let eth_wrapper = Arc::new(HandleWrapper::new(ProtocolId::Eth, handle.clone()));
        //let parity_wrapper = Arc::new(HandleWrapper::new(ProtocolId::Parity, handle.clone()));

        // register ethereum handler
        self.network
            .register_protocol(
                eth_wrapper.clone(),
                ProtocolId::Eth.to_protocol_type(),
                &[
                    (
                        EthProtocolVersion::VERSION_64.to_number(),
                        EthProtocolVersion::VERSION_64.to_version_byte(),
                    ),
                ],
            )
            .unwrap_or_else(|e| error!("Error registering ethereum protocol: {:?}", e));

        // register parity handler
        // self.network
        //     .register_protocol(
        //         parity_wrapper.clone(),
        //         ProtocolId::Parity.to_protocol_type(),
        //         &[
        //             (
        //                 ParityProtocolVersion::VERSION_1 as u8,
        //                 ParityProtocolVersion::VERSION_1.to_version_byte(),
        //             ),
        //             (
        //                 ParityProtocolVersion::VERSION_2 as u8,
        //                 ParityProtocolVersion::VERSION_2.to_version_byte(),
        //             ),
        //         ],
        //     )
        //     .unwrap_or_else(|e| error!("Error registering parity protocol: {:?}", e));
    }

    fn send_mesage(&self, protocol: ProtocolId, peer: &PeerId, mesage_id: u8, data: &[u8]) {
        self.network.with_context_eval(
            protocol.to_protocol_type(),
            |context: &dyn NetworkContext| {
                context.send_protocol(
                    protocol.to_protocol_type(),
                    *peer,
                    mesage_id,
                    Vec::from(data),
                )
            },
        );
    }

    fn penalize_peer(&self, peer: &PeerId, penal: PeerPenal) {
        match penal {
            PeerPenal::Kick => {
                self.network
                    .with_context(*b"   ", |context: &dyn NetworkContext| {
                        context.disconnect_peer(*peer);
                    })
            }
            PeerPenal::Ban => self
                .network
                .with_context(*b"   ", |context: &dyn NetworkContext| {
                    context.disable_peer(*peer);
                }),
        };
    }
}
