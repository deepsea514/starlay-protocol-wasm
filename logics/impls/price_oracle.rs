// Copyright 2023 Asynmatrix Pte. Ltd.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::traits::pool::PoolRef;
pub use crate::traits::price_oracle::*;
use openbrush::{
    contracts::ownable::*,
    storage::Mapping,
    traits::{
        AccountId,
        Storage,
    },
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);
#[derive(Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    /// Fixed prices to behave as Mock
    pub fixed_prices: Mapping<AccountId, u128>,
    /// Authorized sybils for updating Price
    pub sybils: Mapping<AccountId, bool>,
}

pub const PRICE_PRECISION: u128 = 10_u128.pow(18);

pub trait Internal {
    fn _get_price(&self, asset: AccountId) -> Option<u128>;
    fn _get_underlying_price(&self, pool: AccountId) -> Option<u128>;
    fn _set_fixed_price(&mut self, asset: AccountId, price: u128) -> Result<()>;
    fn _authorize_sybil(&mut self, sybil: AccountId) -> Result<()>;
    fn _unauthorize_sybil(&mut self, sybil: AccountId) -> Result<()>;
}

impl<T: Storage<Data> + Storage<ownable::Data>> PriceOracle for T {
    default fn get_price(&self, asset: AccountId) -> Option<u128> {
        self._get_price(asset)
    }
    default fn get_underlying_price(&self, pool: AccountId) -> Option<u128> {
        self._get_underlying_price(pool)
    }
    default fn set_fixed_price(&mut self, asset: AccountId, value: u128) -> Result<()> {
        self._set_fixed_price(asset, value)
    }
    default fn authorize_sybil(&mut self, sybil: AccountId) -> Result<()> {
        self._authorize_sybil(sybil)
    }
    default fn unauthorize_sybil(&mut self, sybil: AccountId) -> Result<()> {
        self._unauthorize_sybil(sybil)
    }
}

impl<T: Storage<Data> + Storage<ownable::Data>> Internal for T {
    default fn _get_price(&self, asset: AccountId) -> Option<u128> {
        self.data::<Data>().fixed_prices.get(&asset)
    }
    default fn _get_underlying_price(&self, pool: AccountId) -> Option<u128> {
        if let Some(underlying) = PoolRef::underlying(&pool) {
            return self._get_price(underlying)
        }
        None
    }
    default fn _set_fixed_price(&mut self, asset: AccountId, value: u128) -> Result<()> {
        self.data::<Data>().fixed_prices.insert(&asset, &value);
        Ok(())
    }
    default fn _authorize_sybil(&mut self, sybil: AccountId) -> Result<()> {
        self.data::<Data>().sybils.insert(&sybil, &true);
        Ok(())
    }
    default fn _unauthorize_sybil(&mut self, sybil: AccountId) -> Result<()> {
        self.data::<Data>().sybils.insert(&sybil, &false);
        Ok(())
    }
}
