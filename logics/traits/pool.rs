use openbrush::{
    contracts::psp22::PSP22Error,
    traits::{
        AccountId,
        Balance,
    },
};

#[openbrush::wrapper]
pub type PoolRef = dyn Pool;

#[openbrush::trait_definition]
pub trait Pool {
    #[ink(message)]
    fn mint(&mut self, mint_amount: Balance) -> Result<()>;

    #[ink(message)]
    fn redeem(&mut self, redeem_tokens: Balance) -> Result<()>;

    #[ink(message)]
    fn redeem_underlying(&mut self, redeem_amount: Balance) -> Result<()>;

    #[ink(message)]
    fn borrow(&mut self, borrow_amount: Balance) -> Result<()>;

    #[ink(message)]
    fn repay_borrow(&mut self, repay_amount: Balance) -> Result<()>;

    #[ink(message)]
    fn repay_borrow_behalf(&mut self, borrower: AccountId, repay_amount: Balance) -> Result<()>;

    #[ink(message)]
    fn liquidate_borrow(
        &mut self,
        borrower: AccountId,
        repay_amount: Balance,
        collateral: AccountId,
    ) -> Result<()>;

    #[ink(message)]
    fn seize(
        &mut self,
        liquidator: AccountId,
        borrower: AccountId,
        seize_tokens: AccountId,
    ) -> AccountId;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    PSP22(PSP22Error),
}

pub type Result<T> = core::result::Result<T, Error>;