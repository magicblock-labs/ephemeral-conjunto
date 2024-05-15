import * as web3 from '@solana/web3.js'
export * from './consts'

export function proxyConnection() {
  return new web3.Connection('http://127.0.0.1:9899', 'confirmed')
}

export function ephemConnection() {
  return new web3.Connection('http://127.0.0.1:8899', 'confirmed')
}

export async function fundAccount(
  conn: web3.Connection,
  pubkey: web3.PublicKey
) {
  const signature = await conn.requestAirdrop(pubkey, web3.LAMPORTS_PER_SOL)
  await conn.confirmTransaction({
    signature,
    ...(await conn.getLatestBlockhash()),
  })
  return signature
}
