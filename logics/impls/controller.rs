use super::exp_no_err::Exp;
use crate::traits::types::WrappedU256;
pub use crate::traits::{
    controller::*,
    pool::PoolRef,
};
use ink::prelude::vec::Vec;
use openbrush::{
    storage::Mapping,
    traits::{
        AccountId,
        Balance,
        Storage,
        ZERO_ADDRESS,
    },
};
use primitive_types::U256;

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub markets: Vec<AccountId>,
    pub mint_guardian_paused: Mapping<AccountId, bool>,
    pub borrow_guardian_paused: Mapping<AccountId, bool>,
    pub oracle: AccountId,
    pub close_factor_mantissa: WrappedU256,
    pub liquidation_incentive_mantissa: WrappedU256,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            markets: Default::default(),
            mint_guardian_paused: Default::default(),
            borrow_guardian_paused: Default::default(),
            oracle: ZERO_ADDRESS.into(),
            close_factor_mantissa: WrappedU256::from(U256::zero()),
            liquidation_incentive_mantissa: WrappedU256::from(U256::zero()),
        }
    }
}

pub trait Internal {
    fn _mint_allowed(&self, pool: AccountId, minter: AccountId, mint_amount: Balance)
        -> Result<()>;
    fn _mint_verify(
        &self,
        pool: AccountId,
        minter: AccountId,
        mint_amount: Balance,
        mint_tokens: Balance,
    ) -> Result<()>;
    fn _redeem_allowed(
        &self,
        pool: AccountId,
        redeemer: AccountId,
        redeem_amount: Balance,
    ) -> Result<()>;
    fn _redeem_verify(
        &self,
        pool: AccountId,
        redeemer: AccountId,
        redeem_amount: Balance,
        redeem_tokens: Balance,
    ) -> Result<()>;
    fn _borrow_allowed(
        &self,
        pool: AccountId,
        borrower: AccountId,
        borrow_amount: Balance,
    ) -> Result<()>;
    fn _borrow_verify(
        &self,
        pool: AccountId,
        borrower: AccountId,
        borrow_amount: Balance,
    ) -> Result<()>;
    fn _repay_borrow_allowed(
        &self,
        pool: AccountId,
        payer: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
    ) -> Result<()>;
    fn _repay_borrow_verify(
        &self,
        pool: AccountId,
        payer: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        borrower_index: u128,
    ) -> Result<()>;
    fn _liquidate_borrow_allowed(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
    ) -> Result<()>;
    fn _liquidate_borrow_verify(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        seize_tokens: Balance,
    ) -> Result<()>;
    fn _seize_allowed(
        &self,
        pool_collateral: AccountId,
        pool_borrowed: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: Balance,
    ) -> Result<()>;
    fn _seize_verify(
        &self,
        pool_collateral: AccountId,
        pool_borrowed: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: Balance,
    ) -> Result<()>;
    fn _transfer_allowed(
        &self,
        pool: AccountId,
        src: AccountId,
        dst: AccountId,
        transfer_tokens: Balance,
    ) -> Result<()>;
    fn _transfer_verify(
        &self,
        pool: AccountId,
        src: AccountId,
        dst: AccountId,
        transfer_tokens: Balance,
    ) -> Result<()>;
    fn _liquidate_calculate_seize_tokens(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        repay_amount: Balance,
    ) -> Result<Balance>;
    fn _set_price_oracle(&mut self, new_oracle: AccountId) -> Result<()>;
    fn _support_market(&mut self, pool: &AccountId) -> Result<()>;
    fn _set_mint_guardian_paused(&mut self, pool: &AccountId, paused: bool) -> Result<()>;
    fn _set_borrow_guardian_paused(&mut self, pool: &AccountId, paused: bool) -> Result<()>;
    fn _set_close_factor(&mut self, new_close_factor_mantissa: WrappedU256) -> Result<()>;
    fn _set_liquidation_incentive(
        &mut self,
        new_liquidation_incentive_mantissa: WrappedU256,
    ) -> Result<()>;

    // view function
    fn _markets(&self) -> Vec<AccountId>;
    fn _is_listed_market(&self, pool: AccountId) -> bool;
    fn _mint_guardian_paused(&self, pool: AccountId) -> Option<bool>;
    fn _borrow_guardian_paused(&self, pool: AccountId) -> Option<bool>;
    fn _close_factor_mantissa(&self) -> Exp;
    fn _liquidation_incentive_mantissa(&self) -> Exp;

    // event emission
    fn _emit_market_listed_event(&self, pool: AccountId);
}

impl<T: Storage<Data>> Controller for T {
    default fn mint_allowed(
        &self,
        pool: AccountId,
        minter: AccountId,
        mint_amount: Balance,
    ) -> Result<()> {
        self._mint_allowed(pool, minter, mint_amount)
    }

    default fn mint_verify(
        &self,
        pool: AccountId,
        minter: AccountId,
        mint_amount: Balance,
        mint_tokens: Balance,
    ) -> Result<()> {
        self._mint_verify(pool, minter, mint_amount, mint_tokens)
    }

    default fn redeem_allowed(
        &self,
        pool: AccountId,
        redeemer: AccountId,
        redeem_amount: Balance,
    ) -> Result<()> {
        self._redeem_allowed(pool, redeemer, redeem_amount)
    }

    default fn redeem_verify(
        &self,
        pool: AccountId,
        redeemer: AccountId,
        redeem_amount: Balance,
        redeem_tokens: Balance,
    ) -> Result<()> {
        self._redeem_verify(pool, redeemer, redeem_amount, redeem_tokens)
    }

    default fn borrow_allowed(
        &self,
        pool: AccountId,
        borrower: AccountId,
        borrow_amount: Balance,
    ) -> Result<()> {
        self._borrow_allowed(pool, borrower, borrow_amount)
    }

    default fn borrow_verify(
        &self,
        pool: AccountId,
        borrower: AccountId,
        borrow_amount: Balance,
    ) -> Result<()> {
        self._borrow_verify(pool, borrower, borrow_amount)
    }

    default fn repay_borrow_allowed(
        &self,
        pool: AccountId,
        payer: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
    ) -> Result<()> {
        self._repay_borrow_allowed(pool, payer, borrower, repay_amount)
    }

    default fn repay_borrow_verify(
        &self,
        pool: AccountId,
        payer: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        borrower_index: u128,
    ) -> Result<()> {
        self._repay_borrow_verify(pool, payer, borrower, repay_amount, borrower_index)
    }

    default fn liquidate_borrow_allowed(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
    ) -> Result<()> {
        self._liquidate_borrow_allowed(
            pool_borrowed,
            pool_collateral,
            liquidator,
            borrower,
            repay_amount,
        )
    }

    default fn liquidate_borrow_verify(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
        seize_tokens: Balance,
    ) -> Result<()> {
        self._liquidate_borrow_verify(
            pool_borrowed,
            pool_collateral,
            liquidator,
            borrower,
            repay_amount,
            seize_tokens,
        )
    }

    default fn seize_allowed(
        &self,
        pool_collateral: AccountId,
        pool_borrowed: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: Balance,
    ) -> Result<()> {
        self._seize_allowed(
            pool_collateral,
            pool_borrowed,
            liquidator,
            borrower,
            seize_tokens,
        )
    }

    default fn seize_verify(
        &self,
        pool_collateral: AccountId,
        pool_borrowed: AccountId,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: Balance,
    ) -> Result<()> {
        self._seize_verify(
            pool_collateral,
            pool_borrowed,
            liquidator,
            borrower,
            seize_tokens,
        )
    }

    default fn transfer_allowed(
        &self,
        pool: AccountId,
        src: AccountId,
        dst: AccountId,
        transfer_tokens: Balance,
    ) -> Result<()> {
        self._transfer_allowed(pool, src, dst, transfer_tokens)
    }

    default fn transfer_verify(
        &self,
        pool: AccountId,
        src: AccountId,
        dst: AccountId,
        transfer_tokens: Balance,
    ) -> Result<()> {
        self._transfer_verify(pool, src, dst, transfer_tokens)
    }

    default fn liquidate_calculate_seize_tokens(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        repay_amount: Balance,
    ) -> Result<Balance> {
        self._liquidate_calculate_seize_tokens(pool_borrowed, pool_collateral, repay_amount)
    }

    default fn set_price_oracle(&mut self, new_oracle: AccountId) -> Result<()> {
        // TODO: assertion check - ownership
        self._set_price_oracle(new_oracle)
    }

    default fn support_market(&mut self, pool: AccountId) -> Result<()> {
        // TODO: assertion check - ownership
        self._support_market(&pool)?;
        self._emit_market_listed_event(pool);
        Ok(())
    }

    default fn set_mint_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()> {
        // TODO: assertion check - ownership
        self._set_mint_guardian_paused(&pool, paused)
    }

    default fn set_borrow_guardian_paused(&mut self, pool: AccountId, paused: bool) -> Result<()> {
        // TODO: assertion check - ownership
        self._set_borrow_guardian_paused(&pool, paused)
    }

    default fn set_close_factor(&mut self, new_close_factor_mantissa: WrappedU256) -> Result<()> {
        // TODO: assertion check - ownership
        self._set_close_factor(new_close_factor_mantissa)
    }

    default fn set_liquidation_incentive(
        &mut self,
        new_liquidation_incentive_mantissa: WrappedU256,
    ) -> Result<()> {
        // TODO: assertion check - ownership
        self._set_liquidation_incentive(new_liquidation_incentive_mantissa)
    }

    default fn markets(&self) -> Vec<AccountId> {
        self._markets()
    }
    default fn mint_guardian_paused(&self, pool: AccountId) -> Option<bool> {
        self._mint_guardian_paused(pool)
    }
    default fn borrow_guardian_paused(&self, pool: AccountId) -> Option<bool> {
        self._borrow_guardian_paused(pool)
    }
}

impl<T: Storage<Data>> Internal for T {
    default fn _mint_allowed(
        &self,
        pool: AccountId,
        _minter: AccountId,
        _mint_amount: Balance,
    ) -> Result<()> {
        if let Some(true) | None = self._mint_guardian_paused(pool) {
            return Err(Error::MintIsPaused)
        }

        // TODO: keep the flywheel moving

        Ok(())
    }
    default fn _mint_verify(
        &self,
        _pool: AccountId,
        _minter: AccountId,
        _mint_amount: Balance,
        _mint_tokens: Balance,
    ) -> Result<()> {
        Ok(()) // do nothing
    }
    default fn _redeem_allowed(
        &self,
        _pool: AccountId,
        _redeemer: AccountId,
        _redeem_amount: Balance,
    ) -> Result<()> {
        // TODO: assertion check - liquidity check to guard against shortfall

        Ok(())
    }
    default fn _redeem_verify(
        &self,
        _pool: AccountId,
        _redeemer: AccountId,
        _redeem_amount: Balance,
        _redeem_tokens: Balance,
    ) -> Result<()> {
        Ok(()) // do nothing
    }
    default fn _borrow_allowed(
        &self,
        pool: AccountId,
        _borrower: AccountId,
        _borrow_amount: Balance,
    ) -> Result<()> {
        if let Some(true) | None = self._borrow_guardian_paused(pool) {
            return Err(Error::BorrowIsPaused)
        }
        // TODO: assertion check - check to already entry market by borrower
        // TODO: assertion check - check oracle price for underlying asset
        // TODO: assertion check - borrow cap
        // TODO: assertion check - HypotheticalAccountLiquidity

        // TODO: keep the flywheel moving

        Ok(())
    }
    default fn _borrow_verify(
        &self,
        _pool: AccountId,
        _borrower: AccountId,
        _borrow_amount: Balance,
    ) -> Result<()> {
        Ok(()) // do nothing
    }
    default fn _repay_borrow_allowed(
        &self,
        _pool: AccountId,
        _payer: AccountId,
        _borrower: AccountId,
        _repay_amount: Balance,
    ) -> Result<()> {
        // TODO: keep the flywheel moving

        Ok(())
    }
    default fn _repay_borrow_verify(
        &self,
        _pool: AccountId,
        _payer: AccountId,
        _borrower: AccountId,
        _repay_amount: Balance,
        _borrower_index: u128,
    ) -> Result<()> {
        Ok(()) // do nothing
    }
    default fn _liquidate_borrow_allowed(
        &self,
        pool_borrowed: AccountId,
        pool_collateral: AccountId,
        _liquidator: AccountId,
        borrower: AccountId,
        repay_amount: Balance,
    ) -> Result<()> {
        if !self._is_listed_market(pool_borrowed) || !self._is_listed_market(pool_collateral) {
            return Err(Error::MarketNotListed)
        }

        // TODO: calculate account's liquidity
        //   The borrower must have shortfall in order to be liquidatable

        //   The liquidator may not repay more than what is allowed by the closeFactor
        let bollow_balance = PoolRef::borrow_balance_stored(&pool_borrowed, borrower);
        let max_close = self
            ._close_factor_mantissa()
            .mul_scalar_truncate(U256::from(bollow_balance));
        if U256::from(repay_amount).gt(&max_close) {
            return Err(Error::TooMuchRepay)
        }

        Ok(())
    }
    default fn _liquidate_borrow_verify(
        &self,
        _pool_borrowed: AccountId,
        _pool_collateral: AccountId,
        _liquidator: AccountId,
        _borrower: AccountId,
        _repay_amount: Balance,
        _seize_tokens: Balance,
    ) -> Result<()> {
        Ok(()) // do nothing
    }
    default fn _seize_allowed(
        &self,
        pool_collateral: AccountId,
        pool_borrowed: AccountId,
        _liquidator: AccountId,
        _borrower: AccountId,
        _seize_tokens: Balance,
    ) -> Result<()> {
        // TODO: assertion check - check paused status

        if !self._is_listed_market(pool_collateral) || !self._is_listed_market(pool_borrowed) {
            return Err(Error::MarketNotListed)
        }
        let p_collateral_ctrler = PoolRef::controller(&pool_collateral);
        let p_borrowed_ctrler = PoolRef::controller(&pool_borrowed);
        if p_collateral_ctrler != p_borrowed_ctrler {
            return Err(Error::ControllerMismatch)
        }

        // TODO: keep the flywheel moving

        Ok(())
    }
    default fn _seize_verify(
        &self,
        _pool_collateral: AccountId,
        _pool_borrowed: AccountId,
        _liquidator: AccountId,
        _borrower: AccountId,
        _seize_tokens: Balance,
    ) -> Result<()> {
        Ok(()) // do nothing
    }
    default fn _transfer_allowed(
        &self,
        _pool: AccountId,
        _src: AccountId,
        _dst: AccountId,
        _transfer_tokens: Balance,
    ) -> Result<()> {
        todo!()
    }
    default fn _transfer_verify(
        &self,
        _pool: AccountId,
        _src: AccountId,
        _dst: AccountId,
        _transfer_tokens: Balance,
    ) -> Result<()> {
        Ok(()) // do nothing
    }
    default fn _liquidate_calculate_seize_tokens(
        &self,
        _pool_borrowed: AccountId,
        _pool_collateral: AccountId,
        _repay_amount: Balance,
    ) -> Result<Balance> {
        todo!()
    }
    default fn _set_price_oracle(&mut self, new_oracle: AccountId) -> Result<()> {
        self.data().oracle = new_oracle;
        Ok(())
    }
    default fn _support_market(&mut self, pool: &AccountId) -> Result<()> {
        self.data().markets.push(*pool);

        // set default states
        self._set_mint_guardian_paused(pool, false)?;
        self._set_borrow_guardian_paused(pool, false)?;

        Ok(())
    }
    default fn _set_mint_guardian_paused(&mut self, pool: &AccountId, paused: bool) -> Result<()> {
        self.data().mint_guardian_paused.insert(pool, &paused);
        Ok(())
    }
    default fn _set_borrow_guardian_paused(
        &mut self,
        pool: &AccountId,
        paused: bool,
    ) -> Result<()> {
        self.data().borrow_guardian_paused.insert(pool, &paused);
        Ok(())
    }
    default fn _set_close_factor(&mut self, new_close_factor_mantissa: WrappedU256) -> Result<()> {
        self.data().close_factor_mantissa = new_close_factor_mantissa;
        Ok(())
    }
    default fn _set_liquidation_incentive(
        &mut self,
        new_liquidation_incentive_mantissa: WrappedU256,
    ) -> Result<()> {
        self.data().liquidation_incentive_mantissa = new_liquidation_incentive_mantissa;
        Ok(())
    }

    default fn _markets(&self) -> Vec<AccountId> {
        self.data().markets.clone()
    }
    default fn _is_listed_market(&self, pool: AccountId) -> bool {
        let markets = self._markets();
        for market in markets {
            if market == pool {
                return true
            }
        }
        return false
    }
    default fn _mint_guardian_paused(&self, pool: AccountId) -> Option<bool> {
        self.data().mint_guardian_paused.get(&pool)
    }
    default fn _borrow_guardian_paused(&self, pool: AccountId) -> Option<bool> {
        self.data().borrow_guardian_paused.get(&pool)
    }
    default fn _close_factor_mantissa(&self) -> Exp {
        Exp {
            mantissa: self.data::<Data>().close_factor_mantissa,
        }
    }
    default fn _liquidation_incentive_mantissa(&self) -> Exp {
        Exp {
            mantissa: self.data::<Data>().liquidation_incentive_mantissa,
        }
    }

    default fn _emit_market_listed_event(&self, _pool: AccountId) {}
}
