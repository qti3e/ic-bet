use ic_cdk::export::candid::CandidType;
use ic_cdk::export::candid::Principal;
use ic_cdk::storage;
use serde::Deserialize;
use std::collections::{hash_map, HashMap};

#[derive(CandidType, Deserialize)]
pub struct Ledger {
    supply: u64,
    balances: HashMap<Principal, u64>,
}

impl Default for Ledger {
    fn default() -> Ledger {
        Ledger {
            supply: 0,
            balances: HashMap::with_capacity(10000),
        }
    }
}

impl Ledger {
    pub fn free(&mut self) {
        self.balances.shrink_to_fit();
    }

    pub fn load(&mut self, data: Ledger) {
        *self = data;
    }

    pub fn balance(account: &Principal) -> u64 {
        let ledger = storage::get::<Ledger>();
        ledger.balances.get(account).cloned().unwrap_or(0)
    }

    pub fn supply() -> u64 {
        let ledger = storage::get::<Ledger>();
        ledger.supply
    }

    pub fn deposit(account: Principal, amount: u64) {
        if amount == 0 {
            return;
        }

        let mut ledger = storage::get_mut::<Ledger>();
        ledger.supply += amount;

        match ledger.balances.entry(account) {
            hash_map::Entry::Vacant(e) => {
                e.insert(amount);
            }
            hash_map::Entry::Occupied(mut e) => {
                *e.get_mut() += amount;
            }
        }
    }

    pub fn withdraw(account: &Principal, amount: u64) -> Result<(), ()> {
        let mut ledger = storage::get_mut::<Ledger>();
        match ledger.balances.get_mut(account) {
            None if amount == 0 => Ok(()),
            None => Err(()),
            Some(balance) if *balance < amount => Err(()),
            Some(balance) => {
                ledger.supply -= amount;
                *balance -= amount;
                Ok(())
            }
        }
    }
}
