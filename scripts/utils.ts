import { Connection, Keypair, PublicKey, Transaction } from '@solana/web3.js'
import {
  mintTo,
  createMint,
  createAssociatedTokenAccount,
  getAssociatedTokenAddress,
} from '@solana/spl-token'
import usdcDummyKp from './usdc_dummy.json'

export interface AnchorWallet {
  publicKey: PublicKey
  signTransaction(transaction: Transaction): Promise<Transaction>
  signAllTransactions(transactions: Transaction[]): Promise<Transaction[]>
}

export class KeypairWallet implements AnchorWallet {
  constructor(readonly payer: Keypair) {}

  async signTransaction(tx: Transaction): Promise<Transaction> {
    tx.partialSign(this.payer)
    return tx
  }

  async signAllTransactions(txs: Transaction[]): Promise<Transaction[]> {
    return txs.map((tx) => {
      tx.partialSign(this.payer)
      return tx
    })
  }

  get publicKey(): PublicKey {
    return this.payer.publicKey
  }
}

// source: Bonfida utils
// https://github.com/Bonfida/bonfida-utils/tree/main/js/src

export const checkAccountExist = async (
  connection: Connection,
  key: PublicKey,
): Promise<boolean> => {
  const info = await connection.getAccountInfo(key)
  if (!info || !info.data) {
    return false
  }
  return true
}

/**
 * A `TokenMint` object can be used to easily create and mint tokens in dev environment
 */
export class TokenMint {
  constructor(
    public token: PublicKey,
    public signer: Keypair,
    public connection: Connection,
  ) {}

  /**
   * Initialize a `TokenMint` object
   * @param connection The solana connection object to the RPC node
   * @param feePayer The fee payer used to create the mint
   * @param mintAuthority The mint authority
   * @returns
   */
  static async init(
    connection: Connection,
    feePayer: Keypair,
    mintAuthority: Keypair,
  ): Promise<TokenMint> {
    const usdcKp = Keypair.fromSecretKey(new Uint8Array(usdcDummyKp))
    let token = await createMint(
      connection,
      feePayer,
      mintAuthority.publicKey,
      null,
      6,
      usdcKp,
    )
    return new TokenMint(token, mintAuthority, connection)
  }

  /**
   * Get or create the associated token account for the specified `wallet`
   * @param wallet The wallet to get the ATA for
   * @param allowOffcurse Allow the owner account to be a PDA
   * @returns
   */
  async getAssociatedTokenAccount(
    wallet: PublicKey,
    allowOffcurse: boolean = false,
  ): Promise<PublicKey> {
    const ata = await getAssociatedTokenAddress(
      this.token,
      wallet,
      allowOffcurse,
    )
    const exist = await checkAccountExist(this.connection, ata)

    if (exist) {
      return ata
    }

    await createAssociatedTokenAccount(
      this.connection,
      this.signer,
      this.token,
      wallet,
    )
    return ata
  }

  /**
   * Mint `amount` tokens into `tokenAccount`
   * @param tokenAccount The token account to mint into
   * @param amount The amount ot mint
   * @returns
   */
  async mintInto(tokenAccount: PublicKey, amount: number): Promise<string> {
    return await mintTo(
      this.connection,
      this.signer,
      this.token,
      tokenAccount,
      this.signer,
      amount,
    )
  }
}

export const getUsdc = async (connection: Connection, authority: Keypair) => {
  // check USDC availability
  const UsdcMainnetPubkey = new PublicKey(
    'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v',
  )
  const UsdcDevnetPubkey = new PublicKey(
    '4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU',
  )

  if (await checkAccountExist(connection, UsdcMainnetPubkey)) {
    return UsdcMainnetPubkey
  } else if (await checkAccountExist(connection, UsdcDevnetPubkey)) {
    return UsdcDevnetPubkey
  } else {
    // initialize dummy USDC, send 1b to authority
    const mint = await TokenMint.init(connection, authority, authority)
    const ata = await mint.getAssociatedTokenAccount(authority.publicKey)
    mint.mintInto(ata, 1_000_000_000_000_000)

    return mint.token
  }
}
