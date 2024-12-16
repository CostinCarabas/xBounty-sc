#[multiversx_sc::module]
pub trait EventsModule {
    #[event("fund")]
    fn fund_event(
        &self,
        #[indexed] repo_owner: ManagedBuffer,
        #[indexed] repo_url: ManagedBuffer,
        #[indexed] issue_id: u64,
        #[indexed] amount: BigUint,
        #[indexed] proposer: ManagedAddress,
    );

    #[event("claim")]
    fn claim_event(
        &self,
        #[indexed] repo_owner: ManagedBuffer,
        #[indexed] repo_url: ManagedBuffer,
        #[indexed] issue_id: u64,
        #[indexed] solver_addr: ManagedAddress,
        #[indexed] solver_github: ManagedBuffer,
    );

    #[event("complete")]
    fn complete_event(
        &self,
        #[indexed] repo_owner: ManagedBuffer,
        #[indexed] repo_url: ManagedBuffer,
        #[indexed] issue_id: u64,
        #[indexed] solver_addr: ManagedAddress,
        #[indexed] solver_github: ManagedBuffer,
        #[indexed] amount: BigUint,
    );
}
