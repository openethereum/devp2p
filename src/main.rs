// Copyright 2020 Gnosis Ltd.
// SPDX-License-Identifier: Apache-2.0

use std::{sync::Arc, thread, time::Duration};

use reth_scheduler::{
    client_adapter::client_info::{Client, Snapshot},
    devp2p_adapter::*,
    Scheduler,
};

mod oe_adapter;
use oe_adapter::*;

extern crate simple_logger;

use log::*;
use simple_logger::SimpleLogger;

pub struct DummyClient();
impl Client for DummyClient {}

pub struct DummySnapshot();
impl Snapshot for DummySnapshot {}

fn main() {
    SimpleLogger::new().with_level(LevelFilter::Info)
        .with_module_level("mio", LevelFilter::Warn)
        .with_module_level("discovery", LevelFilter::Info)
        .with_module_level("network", LevelFilter::Info)
        .init()
        .unwrap();

    info!("Starting organizer");
    let client = Arc::new(DummyClient {});
    let snapshot = Arc::new(DummySnapshot {});
    let devp2p = OeDevp2p::new().unwrap();
    //devp2p.
    let org = Scheduler::new(Box::new(devp2p), client.clone(), snapshot.clone());
    org.start();

    thread::sleep(Duration::from_secs(300));

    org.stop();
}
