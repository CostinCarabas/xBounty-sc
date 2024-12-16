use multiversx_sc::derive_imports::*;
use multiversx_sc::imports::*;

#[type_abi]
#[derive(
    TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, Debug, ManagedVecItem, PartialEq,
)]
pub struct Solver<M: ManagedTypeApi> {
    pub solver_addr: ManagedAddress<M>,
    pub solver_github: ManagedBuffer<M>,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Clone, Debug)]
pub enum BountyStatus {
    Funded,
    Registered,
    Completed,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, Debug)]
pub struct Bounty<M: ManagedTypeApi> {
    pub repo_url: ManagedBuffer<M>,
    pub issue_id: u64,
    pub repo_owner: ManagedBuffer<M>,
    pub amount: BigUint<M>,
    pub proposer: ManagedAddress<M>,
    pub solvers: ManagedVec<M, Solver<M>>,
    pub status: BountyStatus,
    pub created_at: u64,
}

#[multiversx_sc::module]
pub trait StorageModule {
    #[view(getBountyIds)]
    #[storage_mapper("bounties")]
    fn bounties(
        &self,
        repo_owner: &ManagedBuffer,
        repo_url: &ManagedBuffer,
        issue_id: &u64,
    ) -> SingleValueMapper<Bounty<Self::Api>>;

    #[storage_mapper("solvers")]
    fn solvers(&self, solver_addr: &ManagedAddress) -> SingleValueMapper<ManagedBuffer<Self::Api>>;
}
