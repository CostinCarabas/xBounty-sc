#[multiversx_sc::module]
pub trait EventsModule {
    #[event("fund")]
    fn fund_event(
        &self,
        #[indexed] issue_id: u64,
        #[indexed] amount: BigUint,
        #[indexed] proposer: ManagedAddress,
    );

    #[event("claim")]
    fn claim_event(&self, #[indexed] issue_id: u64, #[indexed] solver: ManagedAddress);

    #[event("complete")]
    fn complete_event(
        &self,
        #[indexed] issue_id: u64,
        #[indexed] solver: ManagedAddress,
        #[indexed] amount: BigUint,
    );
}
