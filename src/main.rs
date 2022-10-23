use bitcoin::blockdata::constants::ChainHash;
use bitcoin::hashes::Hash;
use bitcoin::{BlockHash, TxOut};
use clap::Parser;
use lightning::chain::{Access, AccessError};
use lightning::ln::peer_handler::{
    ErroringMessageHandler, IgnoringMessageHandler, MessageHandler, PeerManager,
};
use lightning::routing::gossip::{NetworkGraph, NodeId, P2PGossipSync};
use lightning::util::logger::{Level, Record};
use lightning_net_tokio::{connect_outbound, SocketDescriptor};
use secp256k1::rand::{thread_rng, Rng};
use secp256k1::{PublicKey, SecretKey};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct Logger;
impl lightning::util::logger::Logger for Logger {
    fn log(&self, record: &Record) {
        if record.level >= Level::Warn {
            dbg!(record);
        }
    }
}

struct DummyChainSource;
impl Access for DummyChainSource {
    fn get_utxo(
        &self,
        _genesis_hash: &BlockHash,
        _short_channel_id: u64,
    ) -> Result<TxOut, AccessError> {
        unimplemented!()
    }
}

pub const MAX_SCID_BLOCK: u64 = 0x00ffffff;
pub const MAX_SCID_TX_INDEX: u64 = 0x00ffffff;
pub const MAX_SCID_VOUT_INDEX: u64 = 0xffff;

fn scid_to_string(short_channel_id: u64) -> String {
    let block = (short_channel_id >> 40) as u32;
    let txidx = ((short_channel_id >> 16) & MAX_SCID_TX_INDEX) as u32;
    let outidx = ((short_channel_id) & MAX_SCID_VOUT_INDEX) as u16;

    format!("{}x{}x{}", block, txidx, outidx)
}

#[derive(Debug, Parser)]
struct Opts {
    node_id: PublicKey,
    addr: SocketAddr,
}

#[tokio::main]
async fn main() {
    let opts: Opts = Parser::parse();

    let logger = Arc::new(Logger);

    let chain_hash = ChainHash::using_genesis_block(bitcoin::network::constants::Network::Bitcoin);
    let genesis_block_hash = BlockHash::from_slice(&chain_hash[..]).unwrap();
    let network_graph = Arc::new(NetworkGraph::new(genesis_block_hash, logger.clone()));

    let gossip_sync = Arc::new(P2PGossipSync::<_, Arc<DummyChainSource>, _>::new(
        network_graph.clone(),
        None,
        logger.clone(),
    ));

    let message_handler = MessageHandler {
        chan_handler: ErroringMessageHandler::new(),
        route_handler: gossip_sync.clone(),
        onion_message_handler: IgnoringMessageHandler {},
    };

    let secret_key = SecretKey::new(&mut thread_rng());

    let time = SystemTime::now();
    let timestamp = time.duration_since(UNIX_EPOCH).unwrap().as_secs();

    let ephemeral_random_data: [u8; 32] = thread_rng().gen();

    let peer_manager = Arc::new(PeerManager::<SocketDescriptor, _, _, _, _, _>::new(
        message_handler,
        secret_key,
        timestamp,
        &ephemeral_random_data,
        Arc::new(Logger),
        IgnoringMessageHandler {},
    ));

    connect_outbound(peer_manager.clone(), opts.node_id, opts.addr)
        .await
        .expect("Connecting failed")
        .await;

    println!("Connected!");

    loop {
        println!("Looking for node info");
        if let Some(node) = network_graph
            .read_only()
            .nodes()
            .get(&NodeId::from_pubkey(&opts.node_id))
        {
            if let Some(announcement) = &node.announcement_info {
                println!("Alias = {}", announcement.alias.to_string());
                println!("Features = {}", announcement.features.to_string());
            }
            println!("Channels ({}):", &node.channels.len());
            for channel in &node.channels {
                println!("{}", scid_to_string(*channel));
            }
            break;
        }
        println!("Processing events");
        peer_manager.process_events();
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
