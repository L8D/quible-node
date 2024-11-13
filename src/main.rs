use futures::prelude::stream::StreamExt;
use hex;
use hyper::Method;
use async_trait::async_trait;
use jsonrpsee::core::async_trait as jsonrpsee_async_trait;
use jsonrpsee::types::error::CALL_EXECUTION_FAILED_CODE;
use jsonrpsee::{server::Server, types::ErrorObjectOwned};
use libp2p::{multiaddr, noise, ping, swarm::SwarmEvent, tcp, yamux};
use serde::{Deserialize, Serialize};
use tx::engine::{collect_valid_block_transactions, ExecutionContext};
use tx::types::{Block, BlockHeader, Hashable, Transaction, TransactionOutpoint, TransactionOutput};
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;
use surrealdb::engine::any;
use surrealdb::engine::any::Any as AnyDb;
use surrealdb::error::Db as ErrorDb;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use tokio::{
    select,
    time::{sleep_until, Duration, Instant},
};
use tower_http::cors::{Any, CorsLayer};
use types::HealthCheckResponse;
use db::types::{BlockRow, PendingTransactionRow, SurrealID, TrackerPing};

use rpc::QuibleRpcServer;

pub mod rpc;
pub mod quible_ecdsa_utils;
pub mod quible_transaction_utils;
pub mod tx;
pub mod types;
pub mod db;

const SLOT_DURATION: Duration = Duration::from_secs(4);

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    #[allow(dead_code)]
    id: Thing,
}

pub struct QuibleBlockProposerExecutionContextImpl {
    db: Arc<Surreal<AnyDb>>,
}

#[async_trait]
impl ExecutionContext for QuibleBlockProposerExecutionContextImpl {
    async fn fetch_next_pending_transaction(
        &mut self,
    ) -> anyhow::Result<Option<([u8; 32], Transaction)>> {
        Ok(None)
    }

    async fn fetch_unspent_output(
        &mut self,
        _outpoint: TransactionOutpoint,
    ) -> anyhow::Result<TransactionOutput> {
        todo!("implement output fetching")
    }

    async fn include_in_next_block(
        &mut self,
        _transaction_hash: [u8; 32],
    ) -> anyhow::Result<()> {
        todo!("implement transaction aggregation for block formation")
    }

    async fn record_invalid_transaction(
        &mut self,
        _transaction_hash: [u8; 32],
        _error: anyhow::Error,
    ) -> anyhow::Result<()> {
        todo!("implement transaction invalidation")
    }
}

async fn propose_block(block_number: u64, db_arc: &Arc<Surreal<AnyDb>>) {
    println!("new block! {}", block_number);

    let result = async {
        let previous_block_header: Option<BlockHeader> = db_arc
            .query("SELECT header FROM blocks WHERE height = $height")
            .bind(("height", block_number - 1))
            .await
            .and_then(|mut response| response.take((0, "header")))?;

        let mut execution_context = QuibleBlockProposerExecutionContextImpl {
            db: db_arc.clone()
        };

        let timestamp: u64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| {
                ErrorDb::Thrown(format!("Failed to generate timestamp: {}", e).into())
            })?
            .as_secs();

        collect_valid_block_transactions(&mut execution_context).await?;

        let previous_block_header_hash = previous_block_header.map_or(Ok([0u8; 32]), |h| {
            h.hash()
        })?;

        let block_header = BlockHeader::Version1 {
            previous_block_header_hash,
            merkle_root: [0u8; 32],
            timestamp
        };

        let block_header_hash = block_header.hash()?;
        let block_header_hash_hex = hex::encode(block_header_hash);

        db_arc.create::<Vec<BlockRow>>("blocks")
            .content(BlockRow {
                id: SurrealID(Thing::from((
                    "pending_transactions".to_string(),
                    block_header_hash_hex.clone().to_string(),
                ))),
                hash: block_header_hash_hex,
                header: block_header,
                height: block_number,
                transactions: vec![]
            })
            .await?;

        Ok(()) as Result<(), Box<dyn std::error::Error>>
    }.await;

    if let Err(e) = result {
        eprintln!("Error in propose_block: {:?}", e);
    }
}
pub struct QuibleRpcServerImpl {
    db: Arc<Surreal<AnyDb>>,
    node_signer_key: [u8; 32],
}

#[jsonrpsee_async_trait]
impl rpc::QuibleRpcServer for QuibleRpcServerImpl {
    async fn send_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<(), ErrorObjectOwned> {
        let transaction_hash = transaction.hash().map_err(|err| {
            ErrorObjectOwned::owned::<String>(
                CALL_EXECUTION_FAILED_CODE,
                "call execution failed: failed to compute transaction hash",
                Some(format!("{}", err.root_cause())),
            )
        })?;
        // let transaction_json = serde_json::to_value(&transaction).unwrap();

        let transaction_hash_hex = hex::encode(transaction_hash);
        let result: Result<Vec<PendingTransactionRow>, surrealdb::Error> = self
            .db
            .create("pending_transactions")
            .content(PendingTransactionRow {
                id: SurrealID(Thing::from((
                    "pending_transactions".to_string(),
                    transaction_hash_hex.clone().to_string(),
                ))),

                // TODO: https://linear.app/quible/issue/QUI-99/use-surrealdb-bytes-type-for-storing-hashes
                // hash: surrealdb::sql::Bytes::from(transaction_hash.to_vec()),
                hash: transaction_hash_hex,

                data: transaction.clone(),

                // TODO: https://linear.app/quible/issue/QUI-100/track-transaction-sizes-and-only-pull-small-enough-transactions
                size: 0,
            })
            .await;

        match result {
            Ok(pending_transaction_rows) => {
                if pending_transaction_rows.len() == 0 {
                    Err(ErrorObjectOwned::owned::<String>(
                        CALL_EXECUTION_FAILED_CODE,
                        "call execution failed: transaction already inserted",
                        None,
                    ))
                } else {
                    Ok(())
                }
            }
            Err(error) => Err(ErrorObjectOwned::owned::<String>(
                CALL_EXECUTION_FAILED_CODE,
                "call execution failed: failed to insert",
                Some(error.to_string()),
            )),
        }
    }

    async fn check_health(&self) -> Result<types::HealthCheckResponse, ErrorObjectOwned> {
        Ok(HealthCheckResponse {
            status: "healthy".to_string(),
        })
    }
}

async fn run_derive_server(
    node_signer_key: [u8; 32],
    db: &Arc<Surreal<AnyDb>>,
    port: u16,
) -> anyhow::Result<SocketAddr> {
    let cors = CorsLayer::new()
        // Allow `POST` when accessing the resource
        .allow_methods([Method::POST])
        // Allow requests from any origin
        .allow_origin(Any)
        .allow_headers([hyper::header::CONTENT_TYPE]);
    let middleware = tower::ServiceBuilder::new().layer(cors);

    let server = Server::builder()
        .set_http_middleware(middleware)
        .build(format!("0.0.0.0:{}", port).parse::<SocketAddr>()?)
        .await?;

    let addr = server.local_addr()?;
    let handle = server.start(
        QuibleRpcServerImpl {
            db: db.clone(),
            node_signer_key,
        }
        .into_rpc(),
    );

    tokio::spawn(handle.stopped());

    Ok(addr)
}

// TODO: https://linear.app/quible/issue/QUI-49/refactor-entrypoint-for-easier-unit-testing
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let signing_key_hex = match env::var("QUIBLE_SIGNER_KEY").ok() {
        Some(key) => key,
        None => {
            let key_file_path = env::var("QUIBLE_SIGNER_KEY_FILE")
                .expect("no QUIBLE_SIGNER_KEY or QUIBLE_SIGNER_KEY_FILE provided");

            let contents = fs::read(key_file_path.clone())
                .expect(&format!("failed to read file at {key_file_path}"));
            std::str::from_utf8(&contents).unwrap().trim().to_owned()
        }
    };

    assert!(
        signing_key_hex.clone().len() == 64,
        "unexpected length for QUIBLE_SIGNER_KEY"
    );
    let mut signing_key_decoded = [0u8; 32];
    hex::decode_to_slice(signing_key_hex, &mut signing_key_decoded)?;

    let p2p_port: u16 = env::var("QUIBLE_P2P_PORT")
        .unwrap_or_else(|_| "9014".to_owned())
        .parse()?;

    let rpc_port: u16 = env::var("QUIBLE_RPC_PORT")
        .unwrap_or_else(|_| "9013".to_owned())
        .parse()?;

    let endpoint = env::var("QUIBLE_DATABASE_URL").unwrap_or_else(|_| "memory".to_owned());

    let leader_addr = env::var("QUIBLE_LEADER_MULTIADDR").ok();

    // surrealdb init
    let db = any::connect(endpoint).await?;
    db.use_ns("quible").use_db("quible_node").await?;
    db::schema::initialize_db(&db).await?;

    if let None = leader_addr {
        db::schema::initialize_tracker_db(&db).await?;
    }

    let db_arc = Arc::new(db);
    let server_addr = run_derive_server(signing_key_decoded, &db_arc, rpc_port).await?;
    let url = format!("http://{}", server_addr);
    println!("server listening at {}", url);

    let mut block_number = 0u64;
    let mut block_timestamp = Instant::now();

    let keypair: libp2p_identity::ecdsa::Keypair =
        libp2p_identity::ecdsa::SecretKey::try_from_bytes(signing_key_decoded)?.into();

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair.into())
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_dns()?
        .with_behaviour(|_| ping::Behaviour::default())?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
        .build();

    swarm.listen_on(multiaddr::multiaddr!(Ip4([0, 0, 0, 0]), Tcp(p2p_port)))?;

    let remote_addr = leader_addr
        .clone()
        .map(|url| (url.clone(), url.parse::<multiaddr::Multiaddr>().unwrap()));

    match remote_addr.clone() {
        Some((url, addr)) => {
            match swarm.dial(addr) {
                Err(e) => {
                    eprintln!("Failed to dial {url}: {}", e);
                }

                _ => {}
            };

            println!("Dialed {url}");
        }

        _ => {}
    }

    propose_block(block_number, &db_arc).await;

    loop {
        select! {
            _ = sleep_until(block_timestamp + SLOT_DURATION) => {
                block_timestamp = block_timestamp + SLOT_DURATION;
                block_number += 1;

                propose_block(block_number, &db_arc).await;
            }

            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => println!("libp2p listening on {address:?}"),
                SwarmEvent::Behaviour(libp2p::ping::Event { peer, result: Ok(_), .. }) => {
                    let timestamp: u64 = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_err(|e| {
                            ErrorDb::Thrown(format!("Failed to generate timestamp: {}", e).into())
                        })?
                    .as_secs();

                    db_arc.create::<Vec<TrackerPing>>("tracker_pings")
                        .content(TrackerPing {
                            peer_id: peer.to_base58(),
                            timestamp
                        })
                    .await?;
                },

                // TODO(QUI-46): enable debug log level
                SwarmEvent::OutgoingConnectionError { .. } => {
                    panic!("dial failure: {event:?}");
                },

                SwarmEvent::ConnectionClosed { .. } => {
                    if leader_addr.is_some() {
                        panic!("leader connection closed: {event:?}");
                    }
                },

                _ => {
                    // TODO(QUI-46): enable debug log level
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::types::{BlockRow, PendingTransactionRow};
    use crate::propose_block;
    use crate::rpc::QuibleRpcClient;
    use crate::tx::types::{Hashable, Transaction, TransactionInput, TransactionOutpoint, TransactionOutput};
    use jsonrpsee::http_client::HttpClient;
    use super::{db, run_derive_server};
    use std::sync::Arc;
    use surrealdb::engine::any;

    #[tokio::test]
    async fn test_send_transaction() -> anyhow::Result<()> {
        // Initialize SurrealDB
        let db = any::connect("memory").await?;
        db.use_ns("quible").use_db("quible_node").await?;
        db::schema::initialize_db(&db).await?;

        let db_arc = Arc::new(db);

        let server_addr = run_derive_server(
            hex_literal::hex!("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"),
            &db_arc,
            0,
        )
        .await?;
        let url = format!("http://{}", server_addr);
        println!("server listening at {}", url);
        let client = HttpClient::builder().build(url)?;
        // let signer_secret = k256::ecdsa::SigningKey::random(&mut rand::thread_rng());
        let response = client.send_transaction(
            Transaction::Version1 {
                inputs: vec![],
                outputs: vec![
                    TransactionOutput::Value {
                        value: 0,
                        pubkey_script: vec![],
                    }
                ],
                locktime: 0
            }
        ).await.unwrap();
        dbg!("response: {:?}", response);

        // Query pending transactions from SurrealDB
        let pending_transaction_rows: Vec<PendingTransactionRow> =
            db_arc.select("pending_transactions").await?;

        assert_eq!(pending_transaction_rows.len(), 1);
        for row in pending_transaction_rows {
            println!("Transaction Hash: {}", row.hash);
            println!(
                "Transaction Data: {}",
                serde_json::to_string_pretty(&row.data)?
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_proposes_block_with_no_transactions() -> anyhow::Result<()> {
        // Initialize SurrealDB
        let db = any::connect("memory").await?;
        db.use_ns("quible").use_db("quible_node").await?;
        db::schema::initialize_db(&db).await?;

        let db_arc = Arc::new(db);

        let server_addr = run_derive_server(
            hex_literal::hex!("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"),
            &db_arc,
            0,
        )
        .await?;
        let url = format!("http://{}", server_addr);
        println!("server listening at {}", url);

        propose_block(1, &db_arc).await;

        // Query pending transactions from SurrealDB
        let block_rows: Vec<BlockRow> =
            db_arc.select("blocks").await?;

        assert_eq!(block_rows.len(), 1);
        for row in block_rows {
            println!("Block Hash: {}", row.hash);
            println!(
                "Block Header: {}",
                serde_json::to_string_pretty(&row.header)?
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_accepts_valid_transactions() -> anyhow::Result<()> {
        // Initialize SurrealDB
        let db = any::connect("memory").await?;
        db.use_ns("quible").use_db("quible_node").await?;
        db::schema::initialize_db(&db).await?;

        let db_arc = Arc::new(db);

        let server_addr = run_derive_server(
            hex_literal::hex!("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"),
            &db_arc,
            0,
        )
        .await?;
        let url = format!("http://{}", server_addr);
        println!("server listening at {}", url);
        let client = HttpClient::builder().build(url)?;
        // let signer_secret = k256::ecdsa::SigningKey::random(&mut rand::thread_rng());
        let sample_transaction = Transaction::Version1 {
            inputs: vec![],
            outputs: vec![
                TransactionOutput::Value {
                    value: 0,
                    pubkey_script: vec![],
                }
            ],
            locktime: 0
        };

        let sample_invalid_transaction = Transaction::Version1 {
            inputs: vec![
                TransactionInput {
                    outpoint: TransactionOutpoint {
                        txid: [0u8; 32],
                        index: 0
                    },
                    signature_script: vec![]
                }
            ],
            outputs: vec![
                TransactionOutput::Value {
                    value: 0,
                    pubkey_script: vec![],
                }
            ],
            locktime: 0
        };

        client.send_transaction(sample_transaction.clone()).await.unwrap();
        client.send_transaction(sample_invalid_transaction.clone()).await.unwrap();

        propose_block(1, &db_arc).await;

        // Query pending transactions from SurrealDB
        let block_rows: Vec<BlockRow> =
            db_arc.select("blocks").await?;

        assert_eq!(block_rows.len(), 1);
        for row in block_rows {
            println!("Block Hash: {}", row.hash);
            println!(
                "Block Header: {}",
                serde_json::to_string_pretty(&row.header)?
            );

            assert_eq!(row.transactions.len(), 1);

            for (transaction_hash, _transaction) in row.transactions {
                assert_eq!(transaction_hash, sample_transaction.hash()?);
            }
        }

        Ok(())
    }
}
