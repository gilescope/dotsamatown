use super::polkadot;
use crate::polkadot::RuntimeApi;
use async_std::stream::StreamExt;
use async_trait::async_trait;
use core::future;
use parity_scale_codec::Encode;
// use core::{
//     future::Future,
//     pin::Pin,
//     task::{Context, Poll},
//     time::Duration,
// };
use sp_core::{Decode, H256};
use subxt::{rpc::ClientT, Client, ClientBuilder, DefaultConfig, DefaultExtra};
use futures::{channel::mpsc};
use futures::future::BoxFuture;
#[derive(Encode, Decode)]
pub struct AgnosticBlock {
	pub block_number: u32,
	pub extrinsics: Vec<Vec<u8>>,
}

impl AgnosticBlock {
	pub fn to_vec(&self) -> Vec<u8> {
		self.encode()
	}

	pub fn from_bytes(mut bytes: &[u8]) -> Result<Self, parity_scale_codec::Error> {
		AgnosticBlock::decode(&mut bytes)
	}
}

/// A way to source untransformed raw data.
#[async_trait(?Send)]
pub trait Source {
	// async fn client(&mut self) -> &mut Client<DefaultConfig>;

	// async fn get_api(&mut self) -> &mut RuntimeApi<DefaultConfig, DefaultExtra<DefaultConfig>>;

	async fn fetch_block_hash(
		&mut self,
		block_number: u32,
	) -> Result<Option<sp_core::H256>, BError>;

	async fn fetch_block(
		&mut self,
		block_hash: Option<H256>,
	) -> Result<Option<AgnosticBlock>, BError>;

	async fn fetch_chainname(&mut self) -> Result<Option<String>, BError>;

	async fn fetch_storage(
		&mut self,
		key: subxt::sp_core::storage::StorageKey,
		as_of: Option<H256>,
	) -> Result<Option<subxt::sp_core::storage::StorageData>, BError>;

	async fn fetch_metadata(&mut self, as_of: Option<H256>) -> Result<Option<sp_core::Bytes>, ()>;

	/// We subscribe to relay chains and self sovereign chains
	/// TODO -> impl Iter<BlockHash>
	async fn subscribe_finalised_blocks(
		&mut self,
	) -> Result<
		// Subscription<
		//     subxt::sp_runtime::generic::Header<u32, subxt::sp_runtime::traits::BlakeTwo256>,
		// >
		Box<dyn futures::Stream<Item = Result<H256, ()>> + Unpin>,
		(),
	>;

	fn url(&self) -> &str;
}

pub struct RawDataSource {
	ws_url: String,
	api: Option<RuntimeApi<DefaultConfig, DefaultExtra<DefaultConfig>>>,
}
use futures::future::AbortHandle;
use smoldot_light_wasm::bindings::database_content;
 use smoldot_light_base::Platform;
pub struct SmolDataSource{
	ws_url: String,
}

impl SmolDataSource {
	pub fn new(url: &str) -> Self {
		
    let (new_task_tx, mut new_task_rx) =
        mpsc::unbounded::<(String, BoxFuture<'static, ()>)>();
		let mut client: smoldot_light_base::Client<Vec<futures::future::AbortHandle>, smoldot_light_wasm::platform::Platform> = smoldot_light_base::Client::new(new_task_tx, env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"));

		let json_rpc_running: u32 = 0;

		    // Retrieve the potential relay chains parameter passed through the FFI layer.
		let potential_relay_chains: Vec<smoldot_light_base::ChainId> = vec![
			smoldot_light_base::ChainId::from(1000) //TODO: this is not a relay chain.
		];
		// let potential_relay_chains: Vec<_> = {
		// 	let allowed_relay_chains_ptr = usize::try_from(potential_relay_chains_ptr).unwrap();
		// 	let allowed_relay_chains_len = usize::try_from(potential_relay_chains_len).unwrap();

		// 	let raw_data = unsafe {
		// 		Box::from_raw(core::slice::from_raw_parts_mut(
		// 			allowed_relay_chains_ptr as *mut u8,
		// 			allowed_relay_chains_len * 4,
		// 		))
		// 	};

		// 	raw_data
		// 		.chunks(4)
		// 		.map(|c| u32::from_le_bytes(<[u8; 4]>::try_from(c).unwrap()))
		// 		.map(smoldot_light_base::ChainId::from)
		// 		.collect()
		// };

		// If `json_rpc_running` is non-zero, then we pass a `Sender<String>` to the `add_client`
		// function. The client will push on this channel the JSON-RPC responses and notifications.
		//
		// After the client has pushed a response or notification, we must then propagate it to the
		// FFI layer. This is achieved by spawning a task that continuously polls the `Receiver` (see
		// below).
		//
		// When the chain is later removed, we want the task to immediately stop without sending any
		// additional response or notification to the FFI. This is achieved by storing an
		// `AbortHandle` as the "user data" of the chain within the client. When the chain is removed,
		// the client will yield back this `AbortHandle` and we can use it to abort the task.
		let (json_rpc_responses, responses_rx_and_reg, abort_handle) = if json_rpc_running != 0 {
			let (tx, rx) = mpsc::channel::<String>(64);
			let (handle, reg) = futures::future::AbortHandle::new_pair();
			(Some(tx), Some((rx, reg)), Some(handle))
		} else {
			(None, None, None)
		};

		let polkadot_relay = include_str!("/home/gilescope/git/chainspecs/polkadot/relaychain/chainspec.json");

		let chain_id = client.add_chain(
			smoldot_light_base::AddChainConfig {
                user_data: abort_handle.into_iter().collect(),
                specification: polkadot_relay,
                database_content: "", // start with empty database TODO optimize
                json_rpc_responses,
                potential_relay_chains: potential_relay_chains.into_iter(),
            }
		);
		// smoldot_light_wasm::bindings::init(0,0,0);
// 
		// let chain = client.chains_by_key.values().next().unwrap();



		Self { ws_url: url.to_string() }
	}
}

// struct Platform;

// impl smoldot_light_base::Platform for Platform {

// }


type BError = subxt::GenericError<std::convert::Infallible>; // Box<dyn std::error::Error>;

/// This is the only type that should know about subxt
impl RawDataSource {
	pub fn new(url: &str) -> Self {
		RawDataSource { ws_url: url.to_string(), api: None }
	}

	async fn client(&mut self) -> &mut Client<DefaultConfig> {
		&mut self.get_api().await.client
	}

	async fn get_api(&mut self) -> &mut RuntimeApi<DefaultConfig, DefaultExtra<DefaultConfig>> {
		if self.api.is_some() {
			return self.api.as_mut().unwrap()
		}

		const MAX_RETRIES: usize = 6;
		let mut retries = 0;
		// println!("retries1 {}", retries);
		let client = loop {
			// println!("retries2 {}", retries);
			if retries >= MAX_RETRIES {
				println!("Cannot connect to substrate node after {} retries", retries);
			}
			// println!("retries {}", retries);

			// It might take a while for substrate node that spin up the RPC server.
			// Thus, the connection might get rejected a few times.
			let res = ClientBuilder::new().set_url(&self.ws_url).build().await;

			match res {
				Ok(res) => break res,
				_ => {
					async_std::task::sleep(std::time::Duration::from_secs(1 << retries)).await;
					retries += 1;
				},
			};
		};

		// println!("Client created........................");
		self.api = Some(
			client
				.to_runtime_api::<polkadot::RuntimeApi<DefaultConfig, DefaultExtra<DefaultConfig>>>(
				),
		);
		self.api.as_mut().unwrap()
	}
}

#[async_trait(?Send)]
impl Source for RawDataSource {
	async fn fetch_block_hash(
		&mut self,
		block_number: u32,
	) -> Result<Option<sp_core::H256>, BError> {
		self.get_api().await.client.rpc().block_hash(Some(block_number.into())).await
	}

	/// Return then in bin form rather than link to subxt:
	/// subxt::sp_runtime::generic::SignedBlock<
	///     subxt::sp_runtime::generic::Block<
	///         subxt::sp_runtime::generic::Header<
	///             u32,
	///             subxt::sp_runtime::traits::BlakeTwo256
	///         >,
	///         subxt::sp_runtime::OpaqueExtrinsic
	///       
	async fn fetch_block(
		&mut self,
		block_hash: Option<H256>,
	) -> Result<Option<AgnosticBlock>, BError> {
		let result = self.get_api().await.client.rpc().block(block_hash).await;
		if let Ok(Some(block_body)) = result {
			//TODO: we're decoding and encoding here. cut it out.
			Ok(Some(AgnosticBlock {
				block_number: block_body.block.header.number,
				extrinsics: block_body
					.block
					.extrinsics
					.into_iter()
					.map(|ex| ex.encode())
					.collect::<Vec<_>>(),
			}))
		} else {
			if let Err(err) = result {
				Err(err)
			} else {
				Ok(None)
			}
		}
	}

	async fn fetch_chainname(&mut self) -> Result<Option<String>, BError> {
		self.client().await.rpc().system_chain().await.map(|res| Some(res))
	}

	async fn fetch_storage(
		&mut self,
		key: subxt::sp_core::storage::StorageKey,
		as_of: Option<H256>,
	) -> Result<Option<subxt::sp_core::storage::StorageData>, BError> {
		self.client().await.storage().fetch_raw(key, as_of).await
	}

	async fn fetch_metadata(&mut self, as_of: Option<H256>) -> Result<Option<sp_core::Bytes>, ()> {
		let mut params = None;
		if let Some(hash) = as_of {
			params = Some(jsonrpsee_types::ParamsSer::Array(vec![serde_json::Value::String(
				hex::encode(hash.as_bytes()),
			)]));
		}

		//        self.client().rpc().metadata_bytes(as_of).await
		//TODO: if asof is none then client.Metadata could just be encoded.
		let res = self
			.get_api()
			.await
			.client
			.rpc()
			.client
			.request("state_getMetadata", params.clone())
			.await;
		match res {
			Ok(res) => return Ok(Some(res)),
			_ => return Err(()),
		};
	}

	/// We subscribe to relay chains and self sovereign chains
	async fn subscribe_finalised_blocks(
		&mut self,
	) -> Result<
		// Subscription<
		//     subxt::sp_runtime::generic::Header<u32, subxt::sp_runtime::traits::BlakeTwo256>,
		// >
		Box<dyn futures::Stream<Item = Result<H256, ()>> + Unpin>,
		(),
	> {
		let result = self.get_api().await.client.rpc().subscribe_finalized_blocks().await;
		if let Ok(sub) = result {
			// sub is a Stream... can we map a stream?
			Ok(Box::new(sub.map(|block_header_result| {
				if let Ok(block_header) = block_header_result {
					let block_header: subxt::sp_runtime::generic::Header<
						u32,
						subxt::sp_runtime::traits::BlakeTwo256,
					> = block_header;
					Ok(block_header.hash())
				} else {
					Err(())
				}
			})))
		} else {
			Err(())
		}
	}

	fn url(&self) -> &str {
		&self.ws_url
	}
}
