use openbrush::traits::{
    AccountId,
    Balance,
    Timestamp,
    ZERO_ADDRESS,
};
use primitive_types::U256;
use ink::prelude::vec::Vec;
use super::super::exp_no_err::{
    exp_scale,
    Exp,
};
pub use crate::traits::{
    pool::*,
    price_oracle::PriceOracleRef,
};
use crate::{
    impls::wad_ray_math::{
        exp_ray_ratio,
        Ray,
    },
    traits::{
        controller::ControllerRef,
        math::{
            PercentMath,
            WadRayMath,
        },
        types::WrappedU256,
    },
};
use core::ops::{
    Add,
    Div,
    Mul,
    Sub,
};

pub fn borrow_rate_max_mantissa() -> U256 {
    // .0005% / time
    exp_scale().mul(U256::from(5)).div(U256::from(1000 * 100))
}

pub fn reserve_factor_max_mantissa() -> U256 {
    // 100% / time
    exp_scale()
}

pub fn protocol_seize_share_mantissa() -> U256 {
    exp_scale().mul(U256::from(28)).div(U256::from(10 * 100)) // 2.8%
}

pub struct CalculateInterestInput {
    pub total_borrows: Balance,
    pub total_reserves: Balance,
    pub borrow_index: U256,
    pub borrow_rate: U256,
    pub old_block_timestamp: Timestamp,
    pub new_block_timestamp: Timestamp,
    pub reserve_factor_mantissa: U256,
}

pub struct CalculateInterestOutput {
    pub borrow_index: U256,
    pub total_borrows: Balance,
    pub total_reserves: Balance,
    pub interest_accumulated: Balance,
}

pub const HEALTH_FACTOR_LIQUIDATION_THRESHOLD: u128 = 10_u128.pow(18);

pub fn scaled_amount_of(amount: Balance, idx: Exp) -> Balance {
    let divided = Ray {
        mantissa: WrappedU256::from(U256::from(amount)),
    }
    .ray_div(idx.to_ray())
    .unwrap();
    U256::from(divided.mantissa).as_u128()
}

pub fn from_scaled_amount(scaled_amount: Balance, idx: Exp) -> Balance {
    let multiplied = idx.to_ray().ray_mul(Ray {
        mantissa: WrappedU256::from(U256::from(scaled_amount)),
    });
    U256::from(multiplied.unwrap().mantissa).as_u128()
}

fn compound_interest(borrow_rate_per_millisec: &Exp, delta: U256) -> Exp {
    if delta.is_zero() {
        return Exp {
            mantissa: U256::zero().into(),
        }
    };
    let delta_minus_one = delta.sub(U256::one());
    let delta_minus_two = if delta.gt(&U256::from(2)) {
        delta.sub(U256::from(2))
    } else {
        U256::zero()
    };
    let base_power_two = borrow_rate_per_millisec
        .to_ray()
        .ray_mul(borrow_rate_per_millisec.to_ray())
        .unwrap();
    let base_power_three = base_power_two
        .ray_mul(borrow_rate_per_millisec.to_ray())
        .unwrap();
    let second_term_ray = delta
        .mul(delta_minus_one)
        .mul(U256::from(base_power_two.mantissa))
        .div(U256::from(2));
    let third_term_ray = delta
        .mul(delta_minus_one)
        .mul(delta_minus_two)
        .mul(U256::from(base_power_three.mantissa))
        .div(U256::from(6));

    Exp {
        mantissa: U256::from(borrow_rate_per_millisec.mantissa)
            .mul(delta)
            .add(second_term_ray.div(exp_ray_ratio()))
            .add(third_term_ray.div(exp_ray_ratio()))
            .into(),
    }
}

pub fn calculate_interest(input: &CalculateInterestInput) -> Result<CalculateInterestOutput> {
    if input.borrow_rate.gt(&borrow_rate_max_mantissa()) {
        return Err(Error::BorrowRateIsAbsurdlyHigh)
    }
    let delta = input
        .new_block_timestamp
        .abs_diff(input.old_block_timestamp);
    let compound_interest_factor = compound_interest(
        &Exp {
            mantissa: input.borrow_rate.into(),
        },
        U256::from(delta),
    );

    let interest_accumulated =
        compound_interest_factor.mul_scalar_truncate(U256::from(input.total_borrows));

    let total_borrows_new = interest_accumulated.as_u128().add(input.total_borrows);
    let total_reserves_new = Exp {
        mantissa: WrappedU256::from(input.reserve_factor_mantissa),
    }
    .mul_scalar_truncate_add_uint(interest_accumulated, U256::from(input.total_reserves));
    let borrow_index_new = compound_interest_factor
        .mul_scalar_truncate_add_uint(input.borrow_index.into(), input.borrow_index.into());
    Ok(CalculateInterestOutput {
        borrow_index: borrow_index_new,

        interest_accumulated: interest_accumulated.as_u128(),
        total_borrows: total_borrows_new,
        total_reserves: total_reserves_new.as_u128(),
    })
}

// returns liquidator_seize_tokens, protocol_seize_amount and protocol_seize_tokens
pub fn protocol_seize_amount(
    exchange_rate: Exp,
    seize_tokens: Balance,
    protocol_seize_share_mantissa: U256,
) -> (Balance, Balance, Balance) {
    let protocol_seize_tokens = Exp {
        mantissa: WrappedU256::from(U256::from(seize_tokens).mul(protocol_seize_share_mantissa)),
    }
    .truncate();
    let liquidator_seize_tokens = U256::from(seize_tokens).sub(protocol_seize_tokens);
    (
        liquidator_seize_tokens.as_u128(),
        exchange_rate
            .mul_scalar_truncate(protocol_seize_tokens)
            .as_u128(),
        protocol_seize_tokens.as_u128(),
    )
}

pub fn exchange_rate(
    total_supply: Balance,
    total_cash: Balance,
    total_borrows: Balance,
    total_reserves: Balance,
    default_exchange_rate_mantissa: U256,
) -> U256 {
    if total_supply == 0 {
        return default_exchange_rate_mantissa
    };
    let cash_plus_borrows_minus_reserves = total_cash.add(total_borrows).sub(total_reserves);
    U256::from(cash_plus_borrows_minus_reserves)
        .mul(exp_scale())
        .div(U256::from(total_supply))
}

pub fn balance_decrease_allowed(
    liquidation_threshold: WrappedU256,
    decimals: u8,
    controller: AccountId,
    asset: AccountId,
    user: AccountId,
    amount: Balance,
    oracle: AccountId,
) -> bool {
    let account_assets: Vec<AccountId> = ControllerRef::account_assets(&controller, user);
    if account_assets.is_empty() {
        return true
    }

    if liquidation_threshold == WrappedU256::from(0) {
        return true
    }

    let (total_collateral_in_eth, total_debt_in_eth, _, avg_liquidation_threshold, _) =
        calculate_user_account_data(user, controller, oracle);

    if total_debt_in_eth == U256::from(0) {
        return true
    }

    let asset_price = PriceOracleRef::get_price(&oracle, asset);

    if let None | Some(0) = asset_price {
        return false
    }

    let amount_to_decrease_in_eth = U256::from(asset_price.unwrap())
        .mul(U256::from(amount))
        .div(U256::from(10).pow(U256::from(decimals)));

    let collateral_balance_after_decrease = total_collateral_in_eth.sub(amount_to_decrease_in_eth);

    if collateral_balance_after_decrease == U256::from(0) {
        return false
    }

    let liquidation_threshold_after_decrease = total_collateral_in_eth
        .mul(avg_liquidation_threshold)
        .sub(amount_to_decrease_in_eth.mul(U256::from(liquidation_threshold)))
        .div(collateral_balance_after_decrease);

    let health_factor_after_decrease = calculate_health_factor_from_balances(
        collateral_balance_after_decrease,
        total_debt_in_eth,
        liquidation_threshold_after_decrease,
    );

    health_factor_after_decrease >= U256::from(HEALTH_FACTOR_LIQUIDATION_THRESHOLD)
}

fn calculate_user_account_data(
    user: AccountId,
    controller: AccountId,
    oracle: AccountId,
) -> (U256, U256, U256, U256, U256) {
    let account_assets: Vec<AccountId> = ControllerRef::account_assets(&controller, user);
    if account_assets.is_empty() {
        return (
            U256::from(0),
            U256::from(0),
            U256::from(0),
            U256::from(0),
            U256::MAX,
        )
    }
    let mut account_assets_iter = account_assets.into_iter();

    let markets: Vec<AccountId> = ControllerRef::markets(&controller);
    let mut total_collateral_in_eth = U256::from(0);
    let mut avg_ltv = U256::from(0);
    let mut avg_liquidation_threshold = U256::from(0);
    let mut total_debt_in_eth = U256::from(0);
    for asset in markets.iter() {
        let result = account_assets_iter.find(|&x| x == *asset);
        if result.is_none() || result.unwrap() == ZERO_ADDRESS.into() {
            continue
        }

        let ltv = ControllerRef::collateral_factor_mantissa(&controller, *asset).unwrap();
        let liquidation_threshold = PoolRef::liquidation_threshold(asset);
        let decimals = PoolRef::token_decimals(asset);
        let token_unit = 10_u128.pow(decimals.into());
        let unit_price = PriceOracleRef::get_price(&oracle, *asset).unwrap();

        let (compounded_liquidity_balance, borrow_balance_stored, _) =
            PoolRef::get_account_snapshot(asset, user);

        if liquidation_threshold != WrappedU256::from(0) && compounded_liquidity_balance > 0 {
            let liquidity_balance_eth = U256::from(unit_price)
                .mul(U256::from(compounded_liquidity_balance))
                .div(U256::from(token_unit));
            total_collateral_in_eth = total_collateral_in_eth.add(liquidity_balance_eth);
            avg_ltv = avg_ltv.add(liquidity_balance_eth.mul(U256::from(ltv)));
            avg_liquidation_threshold = avg_liquidation_threshold
                .add(liquidity_balance_eth.mul(U256::from(liquidation_threshold)));
        }

        if borrow_balance_stored != 0 {
            let compounded_borrow_balance = PoolRef::principal_balance_of(asset, user);
            total_debt_in_eth =
                total_debt_in_eth.add(unit_price.mul(compounded_borrow_balance).div(token_unit));
        }
    }

    avg_ltv = if total_collateral_in_eth > U256::from(0) {
        avg_ltv.div(total_collateral_in_eth)
    } else {
        U256::from(0)
    };
    avg_liquidation_threshold = if total_collateral_in_eth > U256::from(0) {
        avg_liquidation_threshold.div(total_collateral_in_eth)
    } else {
        U256::from(0)
    };

    let health_factor = calculate_health_factor_from_balances(
        total_collateral_in_eth,
        total_debt_in_eth,
        avg_liquidation_threshold,
    );
    return (
        total_collateral_in_eth,
        total_debt_in_eth,
        avg_ltv,
        avg_liquidation_threshold,
        health_factor,
    )
}

pub fn calculate_health_factor_from_balances(
    total_collateral_in_eth: U256,
    total_debt_in_eth: U256,
    liquidation_threshold: U256,
) -> U256 {
    if total_debt_in_eth == U256::from(0) {
        return U256::MAX
    }

    total_collateral_in_eth
        .percent_mul(liquidation_threshold)
        .wad_div(total_debt_in_eth)
}

#[cfg(test)]

mod tests {
    use super::Exp;

    use super::*;
    use primitive_types::U256;
    fn mantissa() -> U256 {
        U256::from(10).pow(U256::from(18))
    }

    #[test]
    fn test_scaled_amount_of() {
        struct TestCase {
            amount: Balance,
            idx: Exp,
            want: Balance,
        }
        let cases = vec![
            TestCase {
                amount: 100,
                idx: Exp {
                    mantissa: WrappedU256::from(U256::from(1).mul(mantissa())),
                },
                want: 100,
            },
            TestCase {
                amount: 200,
                idx: Exp {
                    mantissa: WrappedU256::from(U256::from(1).mul(mantissa())),
                },
                want: 200,
            },
            TestCase {
                amount: 100,
                idx: Exp {
                    mantissa: WrappedU256::from(U256::from(100).mul(mantissa())),
                },
                want: 1,
            },
            TestCase {
                amount: 90,
                idx: Exp {
                    mantissa: WrappedU256::from(U256::from(100).mul(mantissa())),
                },
                want: 1,
            },
        ];
        for c in cases {
            assert_eq!(scaled_amount_of(c.amount, c.idx), c.want)
        }
    }
    #[test]
    fn test_calculate_interest_panic_if_over_borrow_rate_max() {
        let input = CalculateInterestInput {
            borrow_index: 0.into(),
            borrow_rate: U256::one().mul(U256::from(10)).pow(U256::from(18)),
            new_block_timestamp: Timestamp::default(),
            old_block_timestamp: Timestamp::default(),
            reserve_factor_mantissa: U256::zero(),
            total_borrows: Balance::default(),
            total_reserves: Balance::default(),
        };
        let out = calculate_interest(&input);
        assert_eq!(out.err().unwrap(), Error::BorrowRateIsAbsurdlyHigh)
    }

    #[test]
    fn test_compound_interest() {
        struct TestInput {
            borrow_rate_per_millisec: Exp,
            delta: U256,
            want: Exp,
        }
        let inputs: &[TestInput] = &[TestInput {
            borrow_rate_per_millisec: Exp {
                mantissa: WrappedU256::from(U256::from(1).mul(mantissa())),
            },
            delta: U256::from(1000_i128 * 60 * 60 * 24 * 30 * 12), // 1 year
            want: Exp {
                mantissa: WrappedU256::from(
                    U256::from(501530650214400000002592_i128)
                        .mul(U256::from(10000000000000000000000000_i128)),
                ),
            },
        }];
        for input in inputs {
            let got = compound_interest(&input.borrow_rate_per_millisec, input.delta);
            assert_eq!(got.mantissa, input.want.mantissa)
        }
    }

    #[test]
    fn test_calculate_apy() {
        // when USDC's utilization rate is 1%
        let utilization_rate_mantissa = mantissa().div(100); // 1%
        let base_rate_per_milli_sec = U256::zero();
        let multiplier_slope_one_mantissa = U256::from(4).mul(mantissa()).div(100); // 4%
        let optimal_utilization_rate_mantissa = U256::from(9).mul(mantissa()).div(10); // 90%
        let multiplier_per_year_slope_one_mantissa = multiplier_slope_one_mantissa
            .mul(mantissa())
            .div(optimal_utilization_rate_mantissa);
        let milliseconds_per_year = U256::from(1000_i128 * 60 * 60 * 24 * 365);
        let multiplier_per_milliseconds_slope_one_mantissa =
            multiplier_per_year_slope_one_mantissa.div(milliseconds_per_year);
        let borrow_rate_mantissa = utilization_rate_mantissa
            .mul(multiplier_per_milliseconds_slope_one_mantissa)
            .div(mantissa())
            .add(base_rate_per_milli_sec);
        let got = compound_interest(
            &Exp {
                mantissa: borrow_rate_mantissa.into(),
            },
            milliseconds_per_year,
        );
        assert_eq!(U256::from(got.mantissa), U256::from(444436848000000_i128))
    }

    #[test]
    fn test_calculate_interest() {
        let old_timestamp = Timestamp::default();
        let inputs: &[CalculateInterestInput] = &[
            CalculateInterestInput {
                old_block_timestamp: old_timestamp,
                new_block_timestamp: old_timestamp + 1000 * 60 * 60 * 24 * 30 * 12, // 1 year
                borrow_index: 1.into(),
                borrow_rate: mantissa().div(100000), // 0.001 %
                reserve_factor_mantissa: mantissa().div(100), // 1 %
                total_borrows: 10_000 * (10_u128.pow(18)),
                total_reserves: 10_000 * (10_u128.pow(18)),
            },
            CalculateInterestInput {
                old_block_timestamp: old_timestamp,
                new_block_timestamp: old_timestamp + 1000 * 60 * 60, // 1 hour
                borrow_index: 123123123.into(),
                borrow_rate: mantissa().div(1000000),
                reserve_factor_mantissa: mantissa().div(10),
                total_borrows: 100_000 * (10_u128.pow(18)),
                total_reserves: 1_000_000 * (10_u128.pow(18)),
            },
            CalculateInterestInput {
                old_block_timestamp: old_timestamp,
                new_block_timestamp: old_timestamp + 1000 * 60 * 60,
                borrow_index: 123123123.into(),
                borrow_rate: mantissa().div(123123),
                reserve_factor_mantissa: mantissa().div(10).mul(2),
                total_borrows: 123_456 * (10_u128.pow(18)),
                total_reserves: 789_012 * (10_u128.pow(18)),
            },
        ];

        for input in inputs {
            let got = calculate_interest(&input).unwrap();
            let delta = input
                .new_block_timestamp
                .abs_diff(input.old_block_timestamp);
            let simple_interest_factor = input.borrow_rate.mul(U256::from(delta));
            let simple_interest_accumulated = simple_interest_factor
                .mul(U256::from(input.total_borrows))
                .div(mantissa())
                .as_u128();
            let reserves_simple = U256::from(input.reserve_factor_mantissa)
                .mul(U256::from(simple_interest_accumulated))
                .div(mantissa())
                .add(U256::from(input.total_reserves));
            assert!(got.interest_accumulated.gt(&simple_interest_accumulated));
            assert!(got
                .total_borrows
                .gt(&(simple_interest_accumulated + (input.total_borrows))));
            assert!(got.total_reserves.gt(&reserves_simple.as_u128()));
            let borrow_idx_simple = U256::from(simple_interest_factor)
                .mul(U256::from(input.borrow_index))
                .div(mantissa())
                .add(U256::from(input.borrow_index));
            assert!(U256::from(got.borrow_index).gt(&borrow_idx_simple));
        }
    }

    #[test]
    // protocol_seize_tokens = seizeTokens * protocolSeizeShare
    // liquidator_seize_tokens = seizeTokens - (seizeTokens * protocolSeizeShare)
    // protocol_seize_amount = exchangeRate * protocolSeizeTokens
    fn test_protocol_seize_amount() {
        // 1%
        let exchange_rate = Exp {
            mantissa: (WrappedU256::from(
                U256::from(10)
                    .pow(U256::from(18))
                    .mul(U256::one())
                    .div(U256::from(100)),
            )),
        };
        let seize_tokens = 10_u128.pow(18).mul(100000000000);
        let protocol_seize_tokens = seize_tokens.mul(10).div(100);
        let protocol_seize_share_mantissa = U256::from(10_u128.pow(18).div(10)); // 10%
        let liquidator_seize_tokens_want = seize_tokens.mul(9).div(10);
        let protocol_seize_amount_want = protocol_seize_tokens.mul(1).div(100); // 1%
        let (liquidator_seize_tokens_got, protocol_seize_amount_got, protocol_seize_tokens_got) =
            protocol_seize_amount(exchange_rate, seize_tokens, protocol_seize_share_mantissa);
        assert_eq!(liquidator_seize_tokens_got, liquidator_seize_tokens_want);
        assert_eq!(protocol_seize_amount_got, protocol_seize_amount_want);
        assert_eq!(protocol_seize_tokens_got, protocol_seize_tokens);
    }
    #[test]
    fn test_exchange_rate_in_case_total_supply_is_zero() {
        let initial = U256::one().mul(exp_scale());
        assert_eq!(exchange_rate(0, 1, 1, 1, initial), initial);
    }

    #[test]
    fn test_exchange_rate() {
        let with_dec = |val: u128| 10_u128.pow(18).mul(val);
        struct Case {
            total_cash: u128,
            total_borrows: u128,
            total_reserves: u128,
            total_supply: u128,
        }
        let cases: &[Case] = &[
            Case {
                total_cash: with_dec(999987),
                total_borrows: with_dec(199987),
                total_reserves: with_dec(299987),
                total_supply: with_dec(1999987),
            },
            Case {
                total_cash: with_dec(999983),
                total_borrows: with_dec(199983),
                total_reserves: with_dec(299983),
                total_supply: with_dec(1999983),
            },
            Case {
                total_cash: with_dec(1999983),
                total_borrows: with_dec(1199983),
                total_reserves: with_dec(1299983),
                total_supply: with_dec(11999983),
            },
            Case {
                total_cash: with_dec(1234567),
                total_borrows: with_dec(234567),
                total_reserves: with_dec(34567),
                total_supply: with_dec(11999983),
            },
        ];
        for case in cases {
            let rate_want = U256::from(10_u128.pow(18))
                .mul(U256::from(
                    case.total_cash + case.total_borrows - case.total_reserves,
                ))
                .div(U256::from(case.total_supply));
            assert_eq!(
                exchange_rate(
                    case.total_supply,
                    case.total_cash,
                    case.total_borrows,
                    case.total_reserves,
                    U256::from(0)
                ),
                rate_want
            )
        }
    }

    #[test]
    fn test_calculate_redeem_values() {
        let mantissa = 10_u128.pow(18);
        struct Input {
            redeem_tokens_in: u128,
            redeem_amount_in: u128,
            exchange_rate: u128,
        }
        struct Output {
            redeem_tokens: u128,
            redeem_amount: u128,
        }
        struct Case {
            input: Input,
            output: Output,
        }
        let cases: &[Case] = &[
            Case {
                input: Input {
                    redeem_tokens_in: 10000,
                    redeem_amount_in: 0,
                    exchange_rate: mantissa * 12 / 10,
                },
                output: Output {
                    redeem_tokens: 10000,
                    redeem_amount: 12000,
                },
            },
            Case {
                input: Input {
                    redeem_tokens_in: 0,
                    redeem_amount_in: 12000,
                    exchange_rate: mantissa * 12 / 10,
                },
                output: Output {
                    redeem_tokens: 10000,
                    redeem_amount: 12000,
                },
            },
        ];
        for case in cases {
            let Case { input, output } = case;
            assert_eq!(
                calculate_redeem_values(
                    input.redeem_tokens_in,
                    input.redeem_amount_in,
                    U256::from(input.exchange_rate)
                ),
                Some((output.redeem_tokens, output.redeem_amount))
            )
        }
    }

    #[test]
    fn test_calculate_redeem_values_is_none() {
        struct Case {
            redeem_tokens_in: u128,
            redeem_amount_in: u128,
        }
        let cases: &[Case] = &[
            Case {
                redeem_tokens_in: 0,
                redeem_amount_in: 0,
            },
            Case {
                redeem_tokens_in: 1,
                redeem_amount_in: 1,
            },
        ];
        for case in cases {
            assert_eq!(
                calculate_redeem_values(
                    case.redeem_tokens_in,
                    case.redeem_amount_in,
                    U256::from(1)
                ),
                None
            )
        }
    }
}
