use crate::log;
use primitive_types::H256;

#[derive(parity_scale_codec::Encode, parity_scale_codec::Decode)]
pub struct AgnosticBlock {
	pub block_number: u32,
	pub extrinsics: Vec<Vec<u8>>,
}

/// A way to source untransformed raw data.
pub trait Source {
	async fn fetch_block_hash(&mut self, block_number: u32) -> Result<Option<H256>, BError>;

	async fn fetch_block(
		&mut self,
		block_hash: Option<H256>,
	) -> Result<Option<AgnosticBlock>, BError>;

	async fn fetch_storage(
		&mut self,
		key: &[u8],
		as_of: Option<H256>,
	) -> Result<Option<Vec<u8>>, BError>;

	async fn fetch_metadata(&mut self, as_of: Option<H256>) -> Result<Option<Vec<u8>>, ()>;

	fn url(&self) -> &str;
}

pub(crate) type WSBackend = polkapipe::ws_web::Backend;

//#[derive(Clone)]
pub struct RawDataSource {
	ws_url: Vec<String>,
	client: Option<polkapipe::PolkaPipe::<WSBackend>>,
}

type BError = polkapipe::Error;

/// This is the only type that should know about subxt
impl RawDataSource {
	pub fn new(url: Vec<String>) -> Self {
		RawDataSource { ws_url: url, client: None }
	}

	async fn client(&mut self) -> Option<&mut polkapipe::PolkaPipe::<WSBackend>> {
		if self.client.is_none() {
			let urls: Vec<_> = self.ws_url.iter().map(|s| s.as_ref()).collect();
			if let Ok(client) = polkapipe::ws_web::Backend::new(urls.as_slice()).await  {
				self.client = Some(polkapipe::PolkaPipe::<polkapipe::ws_web::Backend>{rpc:client});
			}
		}
		self.client.as_mut()
	}
}

impl Source for RawDataSource {
	async fn fetch_block_hash(
		&mut self,
		block_number: u32,
	) -> Result<Option<primitive_types::H256>, BError> {
		// log!("get client");
		if let Some(client) = self.client().await {
			// log!("got client");
			client
				.query_block_hash(&[block_number])
				.await
				.map(|res| Some(H256::from_slice(&res[..])))
		} else {
			log!("could not get client");
			Err(polkapipe::Error::Node(format!("can't get client for {}", self.ws_url[0])))
		}
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
		if let Some(client) = self.client().await {
			let opt = block_hash.map(|b| hex::encode(b.as_bytes()));
			let result = client.query_block(opt.as_deref()).await;

			if let Ok(serde_json::value::Value::Object(map)) = &result {
				// log!("block = {:?}", map);
				if let Some(serde_json::value::Value::Object(map)) = map.get("block") {
					let mut res = AgnosticBlock { block_number: 0, extrinsics: vec![] };
					if let Some(serde_json::value::Value::Object(m)) = map.get("header") {
						// log!("header = {:?}", m);
						if let Some(serde_json::value::Value::String(num_original)) = m.get("number") {
							 let mut num = num_original.trim_start_matches("0x").to_string();
							if num.len() % 2 == 1 {
								// println!("odd found {}", num_original);
								num = format!("0{}", num);
							}

							let _bytes = hex::decode(&num).unwrap();

						//	bytes.reverse(); //TODO: why is this needed? it gets the right number but...
							/* while bytes.len() < 4 {
								bytes.insert(0, 0);
							} */
							// println!("bytes or {}", num_original);
							// println!("bytes is {}", hex::encode(&bytes));
							// use parity_scale_codec::Decode; 

						   let number: u32 = u32::from_str_radix(num_original.trim_start_matches("0x"), 16).unwrap();
//							let number = u32::decode(&mut &bytes[..]).unwrap();

							/* let mut b = [0,0,0,0];
							for (i, bb) in bytes.iter().enumerate() {
								b[i] = *bb;
							} */
							/* use parity_scale_codec::Compact;
					/* 		 */let number = Compact::<u32>::decode(&mut &bytes[..]).unwrap(); */
						/* 	/*  */let re : u32 = number.into(); */
					// println!("bytes {} -> {}",&num_original, number);   
							res.block_number = number;
						}
					}
					if let Some(serde_json::value::Value::Array(extrinsics)) = map.get("extrinsics")
					{
						for ex in extrinsics {
							if let serde_json::value::Value::String(val) = ex {
								/* println!("about to decode '{}'", &val); */
								res.extrinsics
									.push(hex::decode(val.trim_start_matches("0x")).unwrap());
							} else {
								panic!()
							}
						}
						// println!("got 4here aa{}", extrinsics.len());
					}
					return Ok(Some(res))
				}
			}
			result.map(|_| None)
		} else {
			Err(polkapipe::Error::Node(format!("can't get client for {}", self.ws_url[0])))
		}
		// //TODO: we're decoding and encoding here. cut it out.
		// Ok(Some(AgnosticBlock {
		// 	block_number: block_body.block.header.number,
		// 	extrinsics: block_body
		// 		.block
		// 		.extrinsics
		// 		.into_iter()
		// 		.map(|ex| ex.encode())
		// 		.collect::<Vec<_>>(),
		// }))
		// } else {
		// 	if let Err(err) = result {
		// 		Err(err)
		// 	} else {
		// 		Ok(None)
		// 	}
		// }
	}

	async fn fetch_storage(
		&mut self,
		key: &[u8],
		as_of: Option<H256>,
	) -> Result<Option<Vec<u8>>, BError> {
		if let Some(client) = self.client().await {
			if let Some(as_of) = as_of {
				client.query_storage(key, Some(as_of.as_bytes())).await.map(Some)
			} else {
				client.query_storage(key, None).await.map(Some)
			}
		} else {
			Err(polkapipe::Error::Node(format!("can't get client for {}", self.ws_url[0])))
		}
	}

	async fn fetch_metadata(&mut self, as_of: Option<H256>) -> Result<Option<Vec<u8>>, ()> {
		if let Some(client) = self.client().await {
			if let Some(as_of) = as_of {
				client.query_metadata(Some(as_of.as_bytes())).await.map(Some).map_err(|_e| ())
			} else {
				client.query_metadata(None).await.map(Some).map_err(|_e| ())
			}
		} else {
			Err(())
		}
	}

	fn url(&self) -> &str {
		&self.ws_url[0]
	}
}

#[cfg(test)]
mod tests {
	use parity_scale_codec::Encode;

	#[test]
	fn testit() {
		/* let  hex::decode("03ee6c")/*  */.unwrap(); */
		let r: u32 = 10504599;
		let should = r.encode();
		println!("bytes should be {:?}", &should); //bytes [160, 73, 151]

		let bytes = hex::decode("00a04997").unwrap();
		println!("bytes {:?}", &bytes); //bytes [160, 73, 151]
								/* /*  */use parity_scale_codec::Decode; */
		use parity_scale_codec::Decode;
		let mut bytes_rev = bytes.clone();
		bytes_rev.reverse();
		let xg = u32::decode(&mut bytes_rev.as_slice());
		println!("res={:?}.", xg);
		/* /* let x = <u32 as parity_scale_codec::/* Decode */>::decode(&mut
		 * &bytes[..]).unwrap(); */ */
	}
}
