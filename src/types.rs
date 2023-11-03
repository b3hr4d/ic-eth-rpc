use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use cketh_common::eth_rpc::{ProviderError, RpcError};
use cketh_common::eth_rpc_client::providers::{EthereumProvider, SepoliaProvider};

use ic_eth::core::types::RecoveryMessage;
use ic_stable_structures::{BoundedStorable, Storable};
use num_derive::FromPrimitive;
use std::borrow::Cow;
use std::collections::HashMap;

use crate::constants::STRING_STORABLE_MAX_SIZE;
use crate::PROVIDERS;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum Source {
    Url(String),
    Provider(u64),
    Chain(u64),
    Service {
        hostname: String,
        chain_id: Option<u64>,
    },
}

impl Source {
    pub fn resolve(self) -> Result<ResolvedSource, ProviderError> {
        Ok(match self {
            Source::Url(name) => ResolvedSource::Url(name),
            Source::Provider(id) => ResolvedSource::Provider({
                PROVIDERS.with(|providers| {
                    providers
                        .borrow()
                        .get(&id)
                        .ok_or(ProviderError::ProviderNotFound)
                })?
            }),
            Source::Chain(id) => ResolvedSource::Provider(PROVIDERS.with(|providers| {
                let providers = providers.borrow();
                Ok(providers
                    .iter()
                    .find(|(_, p)| p.primary && p.chain_id == id)
                    .or_else(|| providers.iter().find(|(_, p)| p.chain_id == id))
                    .ok_or(ProviderError::ProviderNotFound)?
                    .1)
            })?),
            Source::Service { hostname, chain_id } => {
                ResolvedSource::Provider(PROVIDERS.with(|providers| {
                    let matches_provider = |p: &Provider| {
                        p.hostname == hostname
                            && match chain_id {
                                Some(id) => p.chain_id == id,
                                None => true,
                            }
                    };
                    let providers = providers.borrow();
                    Ok(providers
                        .iter()
                        .find(|(_, p)| p.primary && matches_provider(p))
                        .or_else(|| providers.iter().find(|(_, p)| matches_provider(p)))
                        .ok_or(ProviderError::ProviderNotFound)?
                        .1)
                })?)
            }
        })
    }
}

pub enum ResolvedSource {
    Url(String),
    Provider(Provider),
}

#[derive(Default)]
pub struct Metrics {
    pub requests: u64,
    pub request_cycles_charged: u128,
    pub request_cycles_refunded: u128,
    pub request_err_no_permission: u64,
    pub request_err_host_not_allowed: u64,
    pub request_err_http: u64,
    pub host_requests: HashMap<String, u64>,
}

// These need to be powers of two so that they can be used as bit fields.
#[derive(Clone, Debug, PartialEq, CandidType, FromPrimitive, Deserialize)]
pub enum Auth {
    Admin = 0b0001,
    Rpc = 0b0010,
    RegisterProvider = 0b0100,
    FreeRpc = 0b1000,
}

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub struct Metadata {
    pub nodes_in_subnet: u32,
    pub next_provider_id: u64,
    pub open_rpc_access: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct StringStorable(pub String);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct PrincipalStorable(pub Principal);

impl Storable for StringStorable {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        // String already implements `Storable`.
        self.0.to_bytes()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self(String::from_bytes(bytes))
    }
}

impl BoundedStorable for StringStorable {
    const MAX_SIZE: u32 = STRING_STORABLE_MAX_SIZE;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for PrincipalStorable {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::from(self.0.as_slice())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self(Principal::from_slice(&bytes))
    }
}

impl BoundedStorable for PrincipalStorable {
    const MAX_SIZE: u32 = 29;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ProviderView {
    pub provider_id: u64,
    pub owner: Principal,
    pub chain_id: u64,
    pub hostname: String,
    pub cycles_per_call: u64,
    pub cycles_per_message_byte: u64,
    pub primary: bool,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct RegisterProvider {
    pub chain_id: u64,
    pub hostname: String,
    pub credential_path: String,
    pub cycles_per_call: u64,
    pub cycles_per_message_byte: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct UpdateProvider {
    pub provider_id: u64,
    pub hostname: Option<String>,
    pub credential_path: Option<String>,
    pub cycles_per_call: Option<u64>,
    pub cycles_per_message_byte: Option<u64>,
    pub primary: Option<bool>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Provider {
    pub provider_id: u64,
    pub owner: Principal,
    pub chain_id: u64,
    pub hostname: String,
    pub credential_path: String,
    pub cycles_per_call: u64,
    pub cycles_per_message_byte: u64,
    pub cycles_owed: u128,
    pub primary: bool,
}

impl Provider {
    pub fn service_url(&self) -> String {
        format!("https://{}{}", self.hostname, self.credential_path)
    }
}

impl Storable for Metadata {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(&bytes, Self).unwrap()
    }
}

impl Storable for Provider {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(&bytes, Self).unwrap()
    }
}

impl BoundedStorable for Provider {
    const MAX_SIZE: u32 = 256; // A reasonable limit.
    const IS_FIXED_SIZE: bool = false;
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum Message {
    Data(Vec<u8>),
    Hash([u8; 32]),
}

impl From<Message> for RecoveryMessage {
    fn from(message: Message) -> Self {
        match message {
            Message::Data(d) => RecoveryMessage::Data(d),
            Message::Hash(h) => RecoveryMessage::Hash(h.into()),
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct SignedMessage {
    // TODO: Candid `blob` in place of `vec nat8`
    pub address: Vec<u8>,
    pub message: Message,
    pub signature: Vec<u8>,
}

pub type RpcResult<T> = Result<T, RpcError>;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum CandidRpcSource {
    Ethereum { service: Option<EthereumProvider> },
    Sepolia { service: Option<SepoliaProvider> },
}

pub mod candid_types {
    use std::str::FromStr;

    use candid::CandidType;
    use cketh_common::{
        address::Address,
        eth_rpc::{into_nat, DataFormatError, FixedSizeData},
        eth_rpc_client::responses::TransactionStatus,
        numeric::{BlockNumber, Wei},
    };
    use serde::Deserialize;

    pub use cketh_common::eth_rpc::Hash;

    #[derive(Clone, Debug, CandidType, Deserialize)]
    pub enum BlockSpec {
        Number(u128),
        Tag(BlockTag),
    }

    impl From<BlockSpec> for cketh_common::eth_rpc::BlockSpec {
        fn from(value: BlockSpec) -> Self {
            use cketh_common::eth_rpc::BlockSpec::*;
            match value {
                BlockSpec::Number(n) => Number(n.into()),
                BlockSpec::Tag(t) => Tag(t.into()),
            }
        }
    }

    #[derive(Clone, Debug, CandidType, Deserialize, Default)]
    pub enum BlockTag {
        #[default]
        Latest,
        Finalized,
        Safe,
        Earliest,
        Pending,
        Number(u64),
    }

    impl From<BlockTag> for cketh_common::eth_rpc::BlockTag {
        fn from(value: BlockTag) -> cketh_common::eth_rpc::BlockTag {
            use cketh_common::eth_rpc::BlockTag::*;
            match value {
                BlockTag::Latest => Latest,
                BlockTag::Safe => Safe,
                BlockTag::Finalized => Finalized,
                _ => unimplemented!(),
            }
        }
    }

    #[derive(Clone, Debug, CandidType, Deserialize)]
    pub struct GetLogsArgs {
        pub from_block: Option<BlockSpec>,
        pub to_block: Option<BlockSpec>,
        pub addresses: Vec<String>,
        pub topics: Option<Vec<String>>,
    }

    impl TryFrom<GetLogsArgs> for cketh_common::eth_rpc::GetLogsParam {
        type Error = DataFormatError;
        fn try_from(value: GetLogsArgs) -> Result<Self, Self::Error> {
            Ok(cketh_common::eth_rpc::GetLogsParam {
                from_block: value.from_block.map(|x| x.into()).unwrap_or_default(),
                to_block: value.to_block.map(|x| x.into()).unwrap_or_default(),
                address: value
                    .addresses
                    .into_iter()
                    .map(|s| Address::from_str(&s).map_err(|_| DataFormatError::InvalidHex(s)))
                    .collect::<Result<_, _>>()?,
                topics: value
                    .topics
                    .unwrap_or_default()
                    .into_iter()
                    .map(|s| {
                        FixedSizeData::from_str(&s).map_err(|_| DataFormatError::InvalidHex(s))
                    })
                    .collect::<Result<_, _>>()?,
            })
        }
    }

    #[derive(Clone, Debug, CandidType, Deserialize)]
    pub struct TransactionReceipt {
        pub block_hash: Hash,
        pub block_number: BlockNumber,
        pub effective_gas_price: Wei,
        pub gas_used: candid::Nat,
        pub status: TransactionStatus,
        pub transaction_hash: Hash,
    }

    impl From<cketh_common::eth_rpc_client::responses::TransactionReceipt> for TransactionReceipt {
        fn from(value: cketh_common::eth_rpc_client::responses::TransactionReceipt) -> Self {
            TransactionReceipt {
                block_hash: value.block_hash,
                block_number: value.block_number,
                effective_gas_price: value.effective_gas_price,
                gas_used: into_nat(value.gas_used),
                status: value.status,
                transaction_hash: value.transaction_hash,
            }
        }
    }

    #[derive(Clone, Debug, CandidType, Deserialize)]
    pub struct FeeHistoryArgs {
        pub block_count: u128,
        pub newest_block: BlockSpec,
        pub reward_percentiles: Option<Vec<u8>>,
    }

    impl From<FeeHistoryArgs> for cketh_common::eth_rpc::FeeHistoryParams {
        fn from(value: FeeHistoryArgs) -> Self {
            cketh_common::eth_rpc::FeeHistoryParams {
                block_count: value.block_count.into(),
                highest_block: value.newest_block.into(),
                reward_percentiles: value.reward_percentiles.unwrap_or_default(),
            }
        }
    }

    #[derive(Clone, Debug, CandidType, Deserialize)]
    pub struct GetTransactionCountArgs {
        pub address: String,
        pub block: BlockSpec,
    }

    impl TryFrom<GetTransactionCountArgs>
        for cketh_common::eth_rpc_client::requests::GetTransactionCountParams
    {
        type Error = DataFormatError;
        fn try_from(value: GetTransactionCountArgs) -> Result<Self, Self::Error> {
            Ok(
                cketh_common::eth_rpc_client::requests::GetTransactionCountParams {
                    address: Address::from_str(&value.address)
                        .map_err(|_| DataFormatError::InvalidHex(value.address))?,
                    block: value.block.into(),
                },
            )
        }
    }
}
