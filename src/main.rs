use std::{sync::Arc, thread, time::Duration};

use interfaces::{
    blockchain::BlockchainReadOnly, devp2p::*, importer::Importer, snapshot::Snapshot,
};
use scheduler::Scheduler;

mod dummy_impl;
use dummy_impl::*;
mod oe_adapter;
use oe_adapter::*;

extern crate simple_logger;

use log::*;
use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_module_level("mio", LevelFilter::Warn)
        .with_module_level("discovery", LevelFilter::Info)
        .with_module_level("network", LevelFilter::Info)
        .init()
        .unwrap();

    info!("Starting organizer");
    let blockchain = Arc::new(BlockchainInMemory::new());
    let importer = Arc::new(DummyImporter {});
    let snapshot = Arc::new(DummySnapshot {});
    let devp2p = OeDevp2p::new().unwrap();
    //devp2p.
    let org = Scheduler::new(
        Box::new(devp2p),
        blockchain.clone(),
        importer.clone(),
        snapshot.clone(),
    );
    org.start();

    thread::sleep(Duration::from_secs(300));

    org.stop();
}
