import type { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'

import Pool_factory from '../types/constructors/pool'
import PSP22Token_factory from '../types/constructors/psp22_token'

import Pool from '../types/contracts/pool'
import PSP22Token from '../types/contracts/psp22_token'

type FactoryArgs = {
  api: ApiPromise
  signer: KeyringPair
}

export const deployPool = async ({
  api,
  signer,
  args,
}: FactoryArgs & {
  args: Parameters<Pool_factory['new']>
}): Promise<Pool> => {
  const factory = new Pool_factory(api, signer)
  const contract = await factory.new(...args)
  return new Pool(contract.address, signer, api)
}

// Mocks
export const deployPSP22Token = async ({
  api,
  signer,
  args,
}: FactoryArgs & {
  args: Parameters<PSP22Token_factory['new']>
}): Promise<PSP22Token> => {
  const factory = new PSP22Token_factory(api, signer)
  const contract = await factory.new(...args)
  return new PSP22Token(contract.address, signer, api)
}
