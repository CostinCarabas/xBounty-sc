#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;
use storage::Bounty;
use storage::BountyStatus;
use storage::Solver;

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
            repo_owner: repo_owner.clone(),
            amount: payment_amount.clone(),
            proposer: caller.clone(),
            solvers: ManagedVec::new(),
            status: BountyStatus::Funded,
            created_at: current_timestamp,
        };

        bounties_mapper.set(&bounty);

        // Emit event for funding
        self.fund_event(repo_owner, repo_url, issue_id, payment_amount, caller);
    }

    #[endpoint]
    fn register(
        &self,
        repo_owner: ManagedBuffer,
        repo_url: ManagedBuffer,
        issue_id: u64,
        solver_github: ManagedBuffer,
    ) {
        let caller = self.blockchain().get_caller();
        let bounties_mapper = self.bounties(&repo_owner, &repo_url, &issue_id);
        require!(!bounties_mapper.is_empty(), "Bounty does not exist");

        let mut bounty = bounties_mapper.get();
        require!(
            bounty.status == BountyStatus::Funded,
            "Bounty is not in funded status"
        );

        bounty.solvers.push(Solver {
            solver_addr: caller.clone(),
            solver_github: solver_github.clone(),
        });
        bounty.status = BountyStatus::Registered;

        bounties_mapper.set(&bounty);

        // Emit event for claim
        self.claim_event(repo_owner, repo_url, issue_id, caller, solver_github);
    }

    #[endpoint(releaseBounty)]
    fn release_bounty(
        &self,
        repo_owner: ManagedBuffer,
        repo_url: ManagedBuffer,
        issue_id: u64,
        solver_addr: ManagedAddress,
        solver_github: ManagedBuffer,
    ) {
        let bounties_mapper = self.bounties(&repo_owner, &repo_url, &issue_id);
        require!(!bounties_mapper.is_empty(), "Bounty does not exist");

        let bounty = bounties_mapper.get();
        require!(
            bounty.status == BountyStatus::Registered,
            "Bounty is not in claimed status"
        );

        let caller = self.blockchain().get_caller();
        require!(
            bounty.proposer == caller,
            "Only proposer can release the bounty"
        );

        let solvers_addrs = bounty.solvers.clone();

        require!(
            solvers_addrs.contains(&Solver {
                solver_addr: solver_addr.clone(),
                solver_github: solver_github.clone()
            }),
            "Solver wasn't previously registered"
        );

        self.send().direct_egld(&solver_addr, &bounty.amount);

        // Update status
        let mut updated_bounty = bounty.clone();
        updated_bounty.status = BountyStatus::Completed;
        bounties_mapper.set(&updated_bounty);

        // Emit event for completion
        self.complete_event(
            repo_owner,
            repo_url,
            issue_id,
            solver_addr,
            solver_github,
            bounty.amount,
        );
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
