/* eslint-disable dot-notation */
import { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { BN, BN_ONE, BN_TEN } from '@polkadot/util'
import { ONE_ETHER } from '../scripts/helper/constants'
import {
  deployController,
  deployDefaultInterestRateModel,
  deployLeverager,
  deployPriceOracle,
  deployWETH,
} from '../scripts/helper/deploy_helper'
import { getGasLimit } from '../scripts/helper/utils'

import Leverager from '../types/contracts/leverager'

import {
  PoolContracts,
  Pools,
  preparePoolsWithPreparedTokens,
} from './testContractHelper'
import { shouldNotRevert, shouldNotRevertWithNetworkGas } from './testHelpers'

const MAX_CALL_WEIGHT = new BN(128_000_000_000).isub(BN_ONE).mul(BN_TEN)
const PROOFSIZE = new BN(2_000_000)
describe('Leverager spec', () => {
  let api: ApiPromise
  let deployer: KeyringPair
  let pools: Pools
  let dai: PoolContracts
  let gasLimit: WeightV2
  let leverager: Leverager

  const setup = async () => {
    const { api, alice: deployer, bob, charlie, django } = globalThis.setup

    const gasLimit = getGasLimit(api, MAX_CALL_WEIGHT, PROOFSIZE)
    const controller = await deployController({
      api,
      signer: deployer,
      args: [deployer.address],
    })
    const priceOracle = await deployPriceOracle({
      api,
      signer: deployer,
      args: [],
    })

    const rateModel = await deployDefaultInterestRateModel({
      api,
      signer: deployer,
      args: [[0], [0], [0], [0]],
    })

    const weth = await deployWETH({
      api,
      signer: deployer,
      args: [],
    })

    const pools = await preparePoolsWithPreparedTokens({
      api,
      controller,
      rateModel,
      signer: deployer,
      wethToken: weth,
      manager: deployer.address,
    })

    const users = [bob, charlie, django]

    // initialize
    await controller.tx.setPriceOracle(priceOracle.address)
    await controller.tx.setCloseFactorMantissa([ONE_ETHER])
    //// for pool
    for (const sym of [pools.dai, pools.weth]) {
      await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
      await controller.tx.supportMarketWithCollateralFactorMantissa(
        sym.pool.address,
        sym.token.address,
        [ONE_ETHER.mul(new BN(90)).div(new BN(100))],
      )
    }

    const leverager = await deployLeverager({
      api,
      signer: deployer,
      args: [deployer.address],
    })

    await leverager.tx.initialize(
      controller.address,
      priceOracle.address,
      weth.address,
    )

    return {
      api,
      deployer,
      pools,
      rateModel,
      controller,
      priceOracle,
      users,
      gasLimit,
      leverager,
    }
  }

  beforeAll(async () => {
    ;({ api, deployer, gasLimit, pools, leverager } = await setup())
    ;({ dai } = pools)
  })

  it('.loop', async () => {
    const depositAmount = 2_000
    await shouldNotRevert(dai.token, 'mint', [
      deployer.address,
      depositAmount,
      { gasLimit },
    ])
    await shouldNotRevert(dai.token, 'approve', [
      leverager.address,
      depositAmount,
    ])
    await shouldNotRevert(dai.pool, 'approveDelegate', [
      leverager.address,
      ONE_ETHER,
    ])

    await shouldNotRevertWithNetworkGas(api, leverager, 'loopAsset', [
      dai.token.address,
      depositAmount,
      5000,
      2,
    ])

    const borrowTotal = 1500
    const borrowBalance = (
      await dai.pool.query.borrowBalanceStored(deployer.address)
    ).value.ok
    expect(borrowBalance.toNumber()).toEqual(borrowTotal)

    const depositedUser = (await dai.pool.query.balanceOf(deployer.address))
      .value.ok
    expect(depositedUser.toNumber()).toEqual(borrowTotal + depositAmount)
  })
})
