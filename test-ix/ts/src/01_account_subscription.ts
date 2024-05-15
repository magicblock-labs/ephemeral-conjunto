import test from 'tape'
import { proxyConnection, SOLX_PUBKEY } from './utils'

let conn = proxyConnection()

test('account subscription', async () => {
  // Run a command similar to the below from the terminal for now:
  // solana airdrop -u 'http://127.0.0.1:8899' 1 SoLXmnP9JvL6vJ7TN1VqtTxqsc2izmPfF9CsMDEuRzJ
  const subId = conn.onAccountChange(SOLX_PUBKEY, (account) => {
    console.log('SOLX account changed:', account)
    conn.removeAccountChangeListener(subId)
  })
})
