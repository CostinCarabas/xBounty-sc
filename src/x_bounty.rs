#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;
use storage::Bounty;
use storage::BountyStatus;

mod events;
mod storage;

#[multiversx_sc::contract]
pub trait XBounty: events::EventsModule + storage::StorageModule {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    #[payable("EGLD")]
    #[endpoint]
    fn fund(&self, repo_owner: ManagedBuffer, repo_url: ManagedBuffer, issue_id: u64) {
        let payment_amount = self.call_value().egld_value().clone_value();
        require!(payment_amount > 0, "Payment amount must be greater than 0");

        let bounties_mapper = self.bounties(&repo_owner, &repo_url, &issue_id);

        require!(
            bounties_mapper.is_empty(),
            "Bounty already exists for this issue"
        );

        let caller = self.blockchain().get_caller();
        let current_timestamp = self.blockchain().get_block_timestamp();

        let bounty = Bounty {
            repo_url: repo_url.clone(),
            issue_id,
            repo_owner,
            amount: payment_amount.clone(),
            proposer: caller.clone(),
            solver: None,
            status: BountyStatus::Funded,
            created_at: current_timestamp,
        };

        bounties_mapper.set(&bounty);

        // Emit event for funding
        self.fund_event(repo_url, issue_id, payment_amount, caller);
    }

    #[endpoint]
    fn claim(&self, repo_owner: ManagedBuffer, repo_url: ManagedBuffer, issue_id: u64) {
        let caller = self.blockchain().get_caller();
        let bounties_mapper = self.bounties(&repo_owner, &repo_url, &issue_id);
        require!(!bounties_mapper.is_empty(), "Bounty does not exist");

        let mut bounty = bounties_mapper.get();
        require!(
            bounty.status == BountyStatus::Funded,
            "Bounty is not in funded status"
        );
        require!(bounty.solver.is_none(), "Bounty already has a solver");

        bounty.solver = Some(caller.clone());
        bounty.status = BountyStatus::Claimed;

        bounties_mapper.set(&bounty);

        // Emit event for claim
        self.claim_event(repo_url, issue_id, caller);
    }

    #[endpoint(releaseBounty)]
    fn release_bounty(&self, repo_owner: ManagedBuffer, repo_url: ManagedBuffer, issue_id: u64) {
        let bounties_mapper = self.bounties(&repo_owner, &repo_url, &issue_id);
        require!(!bounties_mapper.is_empty(), "Bounty does not exist");

        let bounty = bounties_mapper.get();
        require!(
            bounty.status == BountyStatus::Claimed,
            "Bounty is not in claimed status"
        );

        let caller = self.blockchain().get_caller();
        require!(
            bounty.proposer == caller,
            "Only proposer can release the bounty"
        );
        require!(bounty.solver.is_some(), "No solver assigned");

        // Send payment to solver
        let solver = bounty.solver.clone().unwrap();
        require!(!solver.is_zero(), "No solver for this bounty");
        self.send().direct_egld(&solver, &bounty.amount);

        // Update status
        let mut updated_bounty = bounty.clone();
        updated_bounty.status = BountyStatus::Completed;
        bounties_mapper.set(&updated_bounty);

        // Emit event for completion
        self.complete_event(repo_url, issue_id, solver, bounty.amount);
    }

    // Views

    #[view(getBounty)]
    fn get_bounty(
        &self,
        repo_owner: ManagedBuffer,
        repo_url: ManagedBuffer,
        issue_id: u64,
    ) -> Option<Bounty<Self::Api>> {
        let bounties_mapper = self.bounties(&repo_owner, &repo_url, &issue_id);
        if bounties_mapper.is_empty() {
            None
        } else {
            Some(bounties_mapper.get())
        }
    }
}
