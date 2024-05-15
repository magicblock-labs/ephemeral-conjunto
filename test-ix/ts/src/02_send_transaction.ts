import test from 'tape'
import web3 from '@solana/web3.js'
import {
  DELEGATED_PUBKEY,
  ephemConnection,
  fundAccount,
  proxyConnection,
} from './utils'

const proxyConn = proxyConnection()
const ephemConn = ephemConnection()

test('send system transfer transaction', async () => {
  const kp = web3.Keypair.generate()
  // 1. Ensure that our accounts exits in the ephemeral validator
  // We use one that is delegated on devnet in order to allow writing
  // to it and verify that this check happens correctly
  // These tests will be much easier to setup once we run a local validator
  // representing devnet/mainnet
  await fundAccount(ephemConn, DELEGATED_PUBKEY)
  await fundAccount(ephemConn, kp.publicKey)

  // 2. Get the latest blockhash from the ephemeral validator (for now)
  // For now we get the latest blockhash from the ephemeral validator since
  // it doesn't properly sync with the chain blockhash yet
  const latestBlockhash = await ephemConn.getLatestBlockhash()

  // 3. Prepare the transfer transaction
  const instructions = [
    web3.SystemProgram.transfer({
      fromPubkey: kp.publicKey,
      toPubkey: DELEGATED_PUBKEY,
      lamports: 111,
    }),
  ]
  const messageV0 = new web3.TransactionMessage({
    payerKey: kp.publicKey,
    recentBlockhash: latestBlockhash.blockhash,
    instructions,
  }).compileToV0Message()
  const tx = new web3.VersionedTransaction(messageV0)
  tx.sign([kp])

  // 4. Send the transaction to the proxy validator (should be handled by ephem validator)
  const signature = await proxyConn.sendTransaction(tx, { skipPreflight: true })
  console.log({ signature })

  // 5. Confirm the transaction
  // NOTE: not working yet as it always passes through to chain
  // await proxyConn.confirmTransaction({
  //   signature,
  //   ...latestBlockhash,
  // })
})
