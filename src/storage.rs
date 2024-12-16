use multiversx_sc::derive_imports::*;
use multiversx_sc::imports::*;

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Clone)]
pub enum BountyStatus {
    Funded,
    Claimed,
    Completed,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct Bounty<M: ManagedTypeApi> {
    pub issue_id: u64,
    pub amount: BigUint<M>,
    pub proposer: ManagedAddress<M>,
    pub solver: Option<ManagedAddress<M>>,
    pub status: BountyStatus,
    pub created_at: u64,
}

#[multiversx_sc::module]
pub trait StorageModule {
    #[view(getBountyIds)]
    #[storage_mapper("bounties")]
    fn bounties(&self, issue_id: &u64) -> SingleValueMapper<Bounty<Self::Api>>;
}
