import {
  AnchorProvider,
  Program,
  setProvider,
  workspace,
} from '@coral-xyz/anchor'
import { Keypair } from '@solana/web3.js'
import { Deezjobs } from '../target/types/deezjobs'
import authorityKp from '../keys/authority.json'
import treasuryKp from '../keys/treasury.json'
import { findProgramAddressSync } from '@coral-xyz/anchor/dist/cjs/utils/pubkey'

setProvider(AnchorProvider.env())

export const program = workspace.Deezjobs as Program<Deezjobs>

export const wait = (ms: number) =>
  new Promise((resolve) => setTimeout(resolve, ms))

export const authority = Keypair.fromSecretKey(new Uint8Array(authorityKp))
export const treasury = Keypair.fromSecretKey(new Uint8Array(treasuryKp))
export const [configPda] = findProgramAddressSync(
  [Buffer.from('config')],
  program.programId,
)
