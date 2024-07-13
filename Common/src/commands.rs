use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Clone, BorshDeserialize, BorshSerialize)]
pub enum MintOperation {
    InitializeMint,
    MintTo(u64),
}
