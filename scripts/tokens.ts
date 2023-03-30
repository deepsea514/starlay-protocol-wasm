import { RiskParameter, RISK_PARAMETERS } from './risk_parameters'
/* eslint-disable @typescript-eslint/naming-convention */
import { BN } from '@polkadot/util'
import { ONE_ETHER } from './helper/constants'
import { InterestRateModel, RATE_MODELS } from './interest_rates'

export interface TokenConfig {
  symbol: string
  decimals: number
  name: string
  rateModel: InterestRateModel
  riskParameter: RiskParameter
  price: BN
}

export interface iAssetBase<T> {
  weth: T
  usdc: T
  usdt: T
  wbtc: T
  wastr: T
  wsdn: T
  dai: T
  busd: T
  matic: T
  bnb: T
  dot: T
}

export type DummyToken = TokenConfig & DummyTokenProp

interface DummyTokenProp {
  totalSupply: BN
}

const TOKEN_BASE: Omit<
  TokenConfig,
  'riskParameter' | 'rateModel' | 'symbol' | 'name'
> = {
  decimals: 18,
  price: ONE_ETHER,
}

export const SUPPORTED_TOKENS: iAssetBase<TokenConfig> = {
  weth: {
    symbol: 'WETH',
    name: 'Wrapped Ether',
    rateModel: RATE_MODELS.weth,
    riskParameter: RISK_PARAMETERS.weth,
    ...TOKEN_BASE,
  },
  bnb: {
    symbol: 'BNB',
    name: 'Binance Coin',
    rateModel: RATE_MODELS.bnb,
    riskParameter: RISK_PARAMETERS.bnb,
    ...TOKEN_BASE,
  },
  dai: {
    symbol: 'DAI',
    name: 'Dai Stablecoin',
    rateModel: RATE_MODELS.dai,
    riskParameter: RISK_PARAMETERS.dai,
    ...TOKEN_BASE,
  },
  usdc: {
    symbol: 'USDC',
    name: 'USD Coin',
    rateModel: RATE_MODELS.usdc,
    riskParameter: RISK_PARAMETERS.usdc,
    decimals: 6,
    ...TOKEN_BASE,
  },
  usdt: {
    symbol: 'USDT',
    name: 'Tether USD',
    rateModel: RATE_MODELS.usdt,
    riskParameter: RISK_PARAMETERS.usdt,
    ...TOKEN_BASE,
    decimals: 6,
  },
  busd: {
    symbol: 'BUSD',
    name: 'Binance USD',
    rateModel: RATE_MODELS.busd,
    riskParameter: RISK_PARAMETERS.busd,
    ...TOKEN_BASE,
  },
  dot: {
    symbol: 'DOT',
    name: 'Polkadot',
    rateModel: RATE_MODELS.busd,
    riskParameter: RISK_PARAMETERS.dot,
    ...TOKEN_BASE,
    decimals: 10,
  },
  matic: {
    symbol: 'MATIC',
    name: 'Matic Token',
    rateModel: RATE_MODELS.matic,
    riskParameter: RISK_PARAMETERS.matic,
    ...TOKEN_BASE,
  },
  wastr: {
    symbol: 'WASTR',
    name: 'Wrapped Astr',
    rateModel: RATE_MODELS.wastr,
    riskParameter: RISK_PARAMETERS.wastr,
    ...TOKEN_BASE,
  },
  wbtc: {
    symbol: 'WBTC',
    name: 'Wrapped Bitcoin',
    rateModel: RATE_MODELS.wbtc,
    riskParameter: RISK_PARAMETERS.wbtc,
    ...TOKEN_BASE,
    decimals: 8,
  },
  wsdn: {
    symbol: 'WSDN',
    name: 'Wrapped SDN',
    rateModel: RATE_MODELS.wsdn,
    riskParameter: RISK_PARAMETERS.wsdn,
    ...TOKEN_BASE,
  },
}

export const DUMMY_TOKENS: DummyToken[] = Object.values(SUPPORTED_TOKENS).map(
  (t) => {
    return {
      ...t,
      totalSupply: new BN(10).pow(new BN(18)).mul(new BN(100_000_000_000)),
    }
  },
)
