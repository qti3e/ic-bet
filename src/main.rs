use crate::ledger::Ledger;
use ic_cdk::*;
use ic_cdk_macros::*;

#[update]
fn balance() -> u64 {
    Ledger::balance(&caller())
}
