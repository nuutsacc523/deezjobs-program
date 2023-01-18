import { Connection, Keypair, PublicKey, Transaction } from '@solana/web3.js'
import {
  mintTo,
  createMint,
  createAssociatedTokenAccount,
  getAssociatedTokenAddress,
} from '@solana/spl-token'
import usdcDummyKp from './usdc_dummy.json'
import { program } from '../tests/common'

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

    const existingToken = await checkAccountExist(connection, usdcKp.publicKey)

    if (existingToken) {
      return new TokenMint(usdcKp.publicKey, mintAuthority, connection)
    }

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
   * @param allowOffCurve Allow the owner account to be a PDA
   * @returns
   */
  async getAssociatedTokenAccount(
    wallet: PublicKey,
    allowOffCurve: boolean = false,
  ): Promise<PublicKey> {
    const ata = await getAssociatedTokenAddress(
      this.token,
      wallet,
      allowOffCurve,
    )
    const exist = await checkAccountExist(this.connection, ata)

    if (exist || allowOffCurve) {
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
  const endpoint = program.provider.connection.rpcEndpoint

  if (endpoint.includes('localhost') || endpoint.includes('127.0.0.1')) {
    // initialize dummy USDC, send 1m to authority
    const mint = await TokenMint.init(connection, authority, authority)
    const ata = await mint.getAssociatedTokenAccount(authority.publicKey)
    mint.mintInto(ata, 1_000_000_000_000)

    return mint.token
  } else if (endpoint.includes('devnet')) {
    return new PublicKey('4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU')
  }

  return new PublicKey('EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v')
}
