#![allow(non_snake_case)]

mod config;
mod proxy;

use config::Config;
use multiversx_sc_snippets::imports::*;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};

const STATE_FILE: &str = "state.toml";

pub async fn x_bounty_cli() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");
    let mut interact = ContractInteract::new().await;
    match cmd.as_str() {
        "deploy" => interact.deploy().await,
        "upgrade" => interact.upgrade().await,
        "fund" => interact.fund().await,
        "register" => interact.register().await,
        "releaseBounty" => interact.release_bounty().await,
        "getBounty" => interact.get_bounty().await,
        "getBountyIds" => interact.bounties().await,
        _ => panic!("unknown command: {}", &cmd),
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    contract_address: Option<Bech32Address>,
}

impl State {
    // Deserializes state from file
    pub fn load_state() -> Self {
        if Path::new(STATE_FILE).exists() {
            let mut file = std::fs::File::open(STATE_FILE).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            toml::from_str(&content).unwrap()
        } else {
            Self::default()
        }
    }

    /// Sets the contract address
    pub fn set_address(&mut self, address: Bech32Address) {
        self.contract_address = Some(address);
    }

    /// Returns the contract address
    pub fn current_address(&self) -> &Bech32Address {
        self.contract_address
            .as_ref()
            .expect("no known contract, deploy first")
    }
}

impl Drop for State {
    // Serializes state to file
    fn drop(&mut self) {
        let mut file = std::fs::File::create(STATE_FILE).unwrap();
        file.write_all(toml::to_string(self).unwrap().as_bytes())
            .unwrap();
    }
}

pub struct ContractInteract {
    interactor: Interactor,
    proposer_address: Address,
    solver_address: Address,
    contract_code: BytesValue,
    state: State,
}

impl ContractInteract {
    pub async fn new() -> Self {
        let config = Config::new();
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());

        interactor.set_current_dir_from_workspace("x_bounty");
        let proposer_address = interactor.register_wallet(test_wallets::alice()).await;
        let solver_address = interactor.register_wallet(test_wallets::bob()).await;

        // Useful in the chain simulator setting
        // generate blocks until ESDTSystemSCAddress is enabled
        interactor.generate_blocks_until_epoch(1).await.unwrap();

        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/x_bounty.mxsc.json",
            &InterpreterContext::default(),
        );

        ContractInteract {
            interactor,
            proposer_address,
            solver_address,
            contract_code,
            state: State::load_state(),
        }
    }

    pub async fn deploy(&mut self) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.proposer_address)
            .gas(30_000_000u64)
            .typed(proxy::XBountyProxy)
            .init()
            .code(&self.contract_code)
            .returns(ReturnsNewAddress)
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state.set_address(Bech32Address::from_bech32_string(
            new_address_bech32.clone(),
        ));

        println!("new address: {new_address_bech32}");
    }

    pub async fn upgrade(&mut self) {
        self.interactor
            .tx()
            .to(self.state.current_address())
            .from(&self.proposer_address)
            .gas(30_000_000u64)
            .typed(proxy::XBountyProxy)
            .upgrade()
            .code(&self.contract_code)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewAddress)
            .run()
            .await;
    }

    pub async fn fund(&mut self) {
        let egld_amount = BigUint::<StaticApi>::from(10u128).pow(17);

        let repo_owner = ManagedBuffer::from("multiversx");
        let repo_url = ManagedBuffer::from("mx-contracts-rs");
        let issue_id = 133u64;

        self.interactor
            .tx()
            .from(&self.proposer_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::XBountyProxy)
            .fund(repo_owner, repo_url, issue_id)
            .egld(egld_amount)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    pub async fn register(&mut self) {
        let repo_owner = ManagedBuffer::from("multiversx");
        let repo_url = ManagedBuffer::from("mx-contracts-rs");
        let issue_id = 133u64;
        let github_id = ManagedBuffer::from("costincarabas");

        self.interactor
            .tx()
            .from(&self.solver_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::XBountyProxy)
            .register(repo_owner, repo_url, issue_id, github_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    pub async fn release_bounty(&mut self) {
        let repo_owner = ManagedBuffer::from("multiversx");
        let repo_url = ManagedBuffer::from("mx-contracts-rs");
        let issue_id = 133u64;
        let github_id = ManagedBuffer::from("costincarabas");

        self.interactor
            .tx()
            .from(&self.proposer_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::XBountyProxy)
            .release_bounty(
                repo_owner,
                repo_url,
                issue_id,
                self.solver_address.clone(),
                github_id,
            )
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    pub async fn get_bounty(&mut self) {
        let repo_owner = ManagedBuffer::new_from_bytes(&b""[..]);
        let repo_url = ManagedBuffer::new_from_bytes(&b""[..]);
        let issue_id = 0u64;

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::XBountyProxy)
            .get_bounty(repo_owner, repo_url, issue_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn bounties(&mut self) {
        let repo_owner = ManagedBuffer::new_from_bytes(&b""[..]);
        let repo_url = ManagedBuffer::new_from_bytes(&b""[..]);
        let issue_id = 0u64;

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::XBountyProxy)
            .bounties(repo_owner, repo_url, issue_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }
}
