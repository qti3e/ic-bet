use crate::ledger::Ledger;
use ic_cdk::export::candid::CandidType;
use ic_cdk::export::candid::Principal;
use serde::Deserialize;
use sha2::{Digest, Sha224};
use std::collections::{hash_map, HashMap};
use std::io::Write;

pub struct Bet {
    started: u64,
    total_bets: u64,
    hasher: Sha224,
    bets: HashMap<(Principal, u8), u64>,
    sum_map: HashMap<u8, u64>,
}

#[derive(CandidType)]
pub struct BetStableBorrowed<'a> {
    started: u64,
    total_bets: u64,
    bets: &'a HashMap<(Principal, u8), u64>,
    sum_map: &'a HashMap<u8, u64>,
}

#[derive(Deserialize)]
pub struct BetStable {
    started: u64,
    total_bets: u64,
    bets: HashMap<(Principal, u8), u64>,
    sum_map: HashMap<u8, u64>,
}

impl Default for Bet {
    fn default() -> Self {
        Bet {
            started: 0,
            total_bets: 0,
            hasher: Sha224::new(),
            bets: HashMap::with_capacity(1000),
            sum_map: HashMap::with_capacity(256),
        }
    }
}

impl Bet {
    pub fn archive(&self) -> BetStableBorrowed {
        BetStableBorrowed {
            started: self.started,
            total_bets: self.total_bets,
            bets: &self.bets,
            sum_map: &self.sum_map,
        }
    }

    pub fn free(&mut self) {
        self.bets.shrink_to_fit();
        self.sum_map.shrink_to_fit();
    }

    pub fn load(&mut self, data: BetStable) {
        self.started = data.started;
        self.total_bets = data.total_bets;
        self.bets = data.bets;
        self.sum_map = data.sum_map;

        if 1000 > self.bets.len() {
            self.bets.reserve(1000 - self.bets.len())
        }

        if 256 > self.sum_map.len() {
            self.sum_map.reserve(256 - self.sum_map.len());
        }
    }

    /// Put a be for a user on a given number.
    pub fn bet(&mut self, user: Principal, number: u8, amount: u64) {
        // Start the time frame by
        if self.started == 0 {
            self.started = now();
        }

        self.hasher
            .write(user.as_slice())
            .expect("Failed to write to hasher.");
        self.hasher
            .write(&[number])
            .expect("Failed to write to hasher.");

        self.total_bets += amount;

        match self.bets.entry((user, number)) {
            hash_map::Entry::Vacant(e) => {
                e.insert(amount);
            }
            hash_map::Entry::Occupied(mut e) => {
                *e.get_mut() += amount;
            }
        }

        match self.sum_map.entry(number) {
            hash_map::Entry::Vacant(e) => {
                e.insert(amount);
            }
            hash_map::Entry::Occupied(mut e) => {
                *e.get_mut() += amount;
            }
        }
    }

    /// Generate the winner number based on the current state of the hasher.
    #[inline]
    fn generate_winner(&mut self) -> u8 {
        let hasher = std::mem::replace(&mut self.hasher, Sha224::new());
        let hash = hasher.finalize();
        let winner = hash[0];
        self.hasher
            .write(hash.as_slice())
            .expect("Failed to write to hasher.");
        winner
    }

    /// Deposit the awards, and return the winning number.
    pub fn close(&mut self) -> u8 {
        let winner = self.generate_winner();

        let winners_total = self.sum_map.get(&winner).cloned().unwrap_or(0);
        let losers_total = self.total_bets - winners_total;

        if winners_total > 0 && losers_total > 0 {
            for ((user, number), amount) in &self.bets {
                if *number == winner {
                    let award = *amount * winners_total / losers_total;
                    Ledger::deposit(user.clone(), award);
                }
            }
        }

        // Reset
        self.total_bets = 0;
        self.bets.clear();
        self.sum_map.clear();

        winner
    }

    #[inline]
    pub fn get_total_bets(&self) -> u64 {
        self.total_bets
    }
}

fn now() -> u64 {
    ic_cdk::api::time() / 1e6
}
