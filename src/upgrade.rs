use crate::bet::{Bet, BetStable, BetStableBorrowed};
use crate::ledger::Ledger;
use ic_cdk::export::candid::{CandidType, Deserialize, Principal};
use ic_cdk::*;
use ic_cdk_macros::*;
use crate::management::Config;

#[derive(Deserialize)]
struct StableStorage {
    config: Config,
    ledger: Ledger,
    bets: BetStable,
}

#[derive(CandidType)]
struct StableStorageBorrowed<'ledger, 'bet, 'config> {
    config: &'config Config,
    ledger: &'ledger Ledger,
    bets: BetStableBorrowed<'bet>,
}

#[pre_upgrade]
pub fn pre_upgrade() {
    let stable = StableStorageBorrowed {
        config: &storage::get::<Config>(),
        ledger: &storage::get::<Ledger>(),
        bets: storage::get::<Bet>().archive(),
    };

    match storage::stable_save((stable,)) {
        Ok(_) => (),
        Err(candid_err) => {
            trap(&format!(
                "An error occurred when saving to stable memory (pre_upgrade): {:?}",
                candid_err
            ));
        }
    };
}

#[post_upgrade]
pub fn post_upgrade() {
    let ledger = storage::get_mut::<Ledger>();
    let bets = storage::get_mut::<Bet>();

    ledger.free();
    bets.free();

    if let Ok((stable,)) = storage::stable_restore::<(StableStorage,)>() {
        ledger.load(stable.ledger);
        bets.load(stable.bets);
        *storage::get_mut::<Config> = stable.config;
    }
}
