import { deployPSP22Token } from './testContractsHelper'
import { hexToUtf8, zeroAddress } from './testHelpers'

import Pool_factory from '../types/constructors/pool'
import Pool from '../types/contracts/pool'

describe('Pool spec', () => {
  const setup = async () => {
    const { api, alice: deployer } = globalThis.setup

    const token = await deployPSP22Token({
      api,
      signer: deployer,
      args: [
        1_000_000,
        'Dai Stablecoin' as unknown as string[],
        'DAI' as unknown as string[],
        8,
      ],
    })

    const poolFactory = new Pool_factory(api, deployer)
    const pool = new Pool(
      (await poolFactory.newFromAsset(token.address)).address,
      deployer,
      api,
    )

    return { token, pool }
  }

  it('instantiate', async () => {
    const { token, pool } = await setup()
    expect(pool.address).not.toBe(zeroAddress)
    expect((await pool.query.underlying()).value.ok).toEqual(token.address)
    expect(hexToUtf8((await pool.query.tokenName()).value.ok)).toEqual(
      'Starlay Dai Stablecoin',
    )
    expect(hexToUtf8((await pool.query.tokenSymbol()).value.ok)).toEqual('sDAI')
    expect((await pool.query.tokenDecimals()).value.ok).toEqual(8)
  })
})
