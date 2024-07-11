use borsh::{BorshDeserialize, BorshSerialize};
use bytes::Bytes;

#[derive(
    Debug, Default, Hash, PartialEq, Eq, Ord, PartialOrd, Clone, BorshDeserialize, BorshSerialize,
)]
pub struct TokenMetadata {
    pub token_id: Bytes,
    pub token_name: Bytes,
    pub decimals: u8,
}
