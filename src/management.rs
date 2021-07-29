use ic_cdk::export::candid::{Principal, CandidType};
use serde::Deserialize;
use ic_cdk::{storage, caller, id, trap, api};
use crate::ledger::Ledger;
use crate::bet::Bet;

#[derive(CandidType, Deserialize)]
pub struct Config {
    controller: Option<Principal>,
    fee_ratio: f64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            controller: None,
            fee_ratio: 4.0,
        }
    }
}

pub struct Controller;

impl Controller {
    pub fn load_if_not_present(user: Principal) {
        let config = storage::get_mut::<Config>();
        if config.controller.is_none() {
            config.controller = Some(user);
        }
    }

    pub fn get_principal() -> Principal {
        let config = storage::get_mut::<Config>();
        config.controller.unwrap().clone()
    }

    #[inline]
    pub fn call_guard() {
        let config = storage::get_mut::<Config>();
        if &caller() != config.controller.unwrap() {
            trap("Only the controller is allowed to call this method.");
        }
    }
}

#[init]
fn init() {
    Controller::load_if_not_present(caller());
}

#[derive(CandidType)]
struct Stats {
    balance: u64,
    captured_fees: u64,
    ledger_supply: u64,
    total_bets: u64,
    previous_number: u64
}

#[update]
fn stats() -> Stats {
    let bet = storage::get::<Bet>();
    Stats {
        balance: api::canister_balance(),
        captured_fees: Ledger::balance(&id()),
        ledger_supply: Ledger::supply(),
        total_bets: bet.get_total_bets(),
        previous_number
    }
}

#[update]
fn send_fees() {
    Controller::call_guard();
}
