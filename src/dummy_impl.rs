// Copyright 2020 Gnosis Ltd.
// SPDX-License-Identifier: Apache-2.0

use interfaces::{
    blockchain::BlockchainReadOnly, devp2p::*, importer::Importer, snapshot::Snapshot,
};
use core::{BlockNumber,BlockId, BlockHeader,BlockBody, H256,BlockReceipt};
use std::{collections::HashMap, sync::Mutex};



pub struct BlockchainInMemory {
    headers: Mutex<HashMap<BlockNumber, BlockHeader>>,
}

impl BlockchainInMemory {
    pub fn new() -> Self {
        BlockchainInMemory {
            headers: Mutex::new(HashMap::new()),
        }
    }
}

impl BlockchainReadOnly for BlockchainInMemory {

    
    fn header(&self, number: BlockNumber) -> Option<BlockHeader> {
        self.headers.lock().unwrap().get(&number).cloned()
    }


    fn body(&self, hash: &H256) -> Option<BlockBody> {
        None
    }

    fn receipt(&self) -> Option<BlockReceipt> {
        unimplemented!()
    }

    fn best_header(&self) -> Option<BlockNumber> {
        self.headers.lock().unwrap().keys().max().cloned()
    }

    // TODO this is just temporary added. Remove when proper header mechanism is build.
    fn import_header(&self, header: &BlockHeader) {
        self.headers.lock().unwrap().insert(header.number, header.clone());
    }


    fn header_list(&self, request: Vec<BlockId>) -> Vec<BlockHeader> {
        todo!()
    }

    fn header_request(
        &self,
        block_id: BlockId,
        max_header: u64,
        skip: u64,
        reverse: bool,
    ) -> Vec<BlockHeader> {
        let mut headers = vec![];
        let mut block_number = match block_id {
            BlockId::Hash(hash) => {
                return headers;
            } // TODO
            BlockId::Number(number) => number,
        };
        while let Some(header) = self.header(block_number) {
            headers.push(header);
            if headers.len() as u64 >= max_header {
                break;
            }
            if reverse {
                block_number -= skip + 1
            } else {
                block_number += skip + 1
            }
        }
        headers
    }

    fn tx(&self) {
        todo!()
    }
}


pub struct DummyImporter();
impl Importer for DummyImporter {}

pub struct DummySnapshot();
impl Snapshot for DummySnapshot {
    fn create_snapshot(&self) {
        todo!()
    }

    fn manifest(&self) {
        todo!()
    }

    fn status(&self) {
        todo!()
    }

    fn chunk(&self) {
        todo!()
    }

    fn begin_restoration(&self, manifest: &interfaces::snapshot::Manifest) {
        todo!()
    }

    fn abort_restoration(&self) {
        todo!()
    }

    fn restore_chunk(&self, chunk: Vec<u8>, chunk_type: interfaces::snapshot::ChunkType) {
        todo!()
    }
}