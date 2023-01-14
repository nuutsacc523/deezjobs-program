import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from '@solana/web3.js'
import { AnchorProvider, BN, Program } from '@coral-xyz/anchor'
import { findProgramAddressSync } from '@coral-xyz/anchor/dist/cjs/utils/pubkey'
import { Deezjobs } from '../target/types/deezjobs'
import { getUsdc, KeypairWallet } from './utils'
import idl from '../target/idl/deezjobs.json'
import programKp from '../target/deploy/deezjobs-keypair.json'
import authorityKp from '../keys/authority.json'
import treasuryKp from '../keys/treasury.json'
import {
  ASSOCIATED_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from '@coral-xyz/anchor/dist/cjs/utils/token'
import { getAssociatedTokenAddress } from '@solana/spl-token'

const { publicKey: programId } = Keypair.fromSecretKey(
  new Uint8Array(programKp),
)
const authority = Keypair.fromSecretKey(new Uint8Array(authorityKp))
const treasury = Keypair.fromSecretKey(new Uint8Array(treasuryKp))

const program = new Program<Deezjobs>(
  idl as unknown as Deezjobs,
  programId,
  new AnchorProvider(
    AnchorProvider.env().connection,
    new KeypairWallet(authority),
    {},
  ),
)

const [programDataPda] = findProgramAddressSync(
  [programId.toBytes()],
  new PublicKey('BPFLoaderUpgradeab1e11111111111111111111111'),
)

const [configPda] = findProgramAddressSync([Buffer.from('config')], programId)

const init = async () => {
  const usdcPubkey = await getUsdc(program.provider.connection, authority)
  const existingConfig = await program.account.config.fetchNullable(configPda)

  if (!existingConfig) {
    try {
      const treasuryUsdcAta = await getAssociatedTokenAddress(
        usdcPubkey,
        treasury.publicKey,
      )

      const accounts = {
        config: configPda,
        treasury: treasury.publicKey,
        program: program.programId,
        programData: programDataPda,
        mint: usdcPubkey,
        treasuryTokenAccount: treasuryUsdcAta,
        upgradeAuthority: authority.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      }

      console.log('Accounts:\n', JSON.stringify(accounts, null, 2))

      await program.methods
        .initialize({
          // USDC has 6 decimal places
          clientFeeMin: new BN(2_000_000),
          // 100_00 as 100%
          clientFeePercentage: 3_00,
          freelancerFeePercentage: 5_00,
          referralFeePercentage: 5_00,
        })
        .accounts(accounts)
        .rpc()

      console.log('Global config initialized')
    } catch (e) {
      console.log(e)
      throw new Error(e)
    }
  }
}

init()
