import { AnchorError } from '@coral-xyz/anchor'
import { findProgramAddressSync } from '@coral-xyz/anchor/dist/cjs/utils/pubkey'
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token'
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from '@solana/web3.js'
import { BN } from 'bn.js'
import { assert } from 'chai'
import { TokenMint } from '../scripts/utils'
import { authority, configPda, program, treasury, wait } from './common'

let freelancer = Keypair.generate()
let client = Keypair.generate()
let gigId = Keypair.generate()
let gigNonce = gigId.publicKey.toBytes().slice(0, 8)
let [gigPda] = findProgramAddressSync(
  [Buffer.from('gig'), freelancer.publicKey.toBytes(), gigNonce],
  program.programId,
)
let [dealPda] = findProgramAddressSync(
  [Buffer.from('deal'), client.publicKey.toBytes(), gigPda.toBytes()],
  program.programId,
)
let dealEscrowPda: PublicKey
let freelancerUsdc: PublicKey
let clientUsdc: PublicKey
let treasuryUsdc: PublicKey
let usdc: TokenMint

describe('Gig & Deal interaction', () => {
  const offer = new BN(100_000_000)
  const deadline = new BN(new Date().getTime() / 1000 + 60 * 24 * 2)

  before(async () => {
    const airdropAmount = 2 * LAMPORTS_PER_SOL

    await program.provider.connection.requestAirdrop(
      freelancer.publicKey,
      airdropAmount,
    )

    await wait(500)

    await program.provider.connection.requestAirdrop(
      client.publicKey,
      airdropAmount,
    )

    await wait(500)

    usdc = await TokenMint.init(
      program.provider.connection,
      authority,
      authority,
    )

    clientUsdc = await usdc.getAssociatedTokenAccount(client.publicKey)
    freelancerUsdc = await usdc.getAssociatedTokenAccount(freelancer.publicKey)
    treasuryUsdc = await usdc.getAssociatedTokenAccount(treasury.publicKey)
    dealEscrowPda = await usdc.getAssociatedTokenAccount(dealPda, true)

    await usdc.mintInto(clientUsdc, 1_000_000_000)
  })

  it('should allow the freelancer to create a gig', async () => {
    const asking = new BN(100_000_000)
    const minCompletionTime = new BN(60 * 24)

    await program.methods
      .createGig({
        asking,
        category: 0,
        skills: new BN(0),
        minCompletionTime,
      })
      .accounts({
        id: gigId.publicKey,
        gig: gigPda,
        mint: usdc.token,
        owner: freelancer.publicKey,
        payer: freelancer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([freelancer, gigId])
      .rpc()

    const gig = await program.account.gig.fetch(gigPda)

    assert.ok(gig.asking.eq(asking))
    assert.ok(gig.minCompletionTime.eq(minCompletionTime))
  })

  it('should allow the client to offer a deal', async () => {
    await program.methods
      .createDeal({
        offer,
        deadline,
        referrer: null,
      })
      .accounts({
        config: configPda,
        deal: dealPda,
        gig: gigPda,
        mint: usdc.token,
        owner: client.publicKey,
        ownerWallet: clientUsdc,
        escrow: dealEscrowPda,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([client])
      .rpc()

    const deal = await program.account.deal.fetch(dealPda)

    const escrowBalance =
      await program.provider.connection.getTokenAccountBalance(dealEscrowPda)

    const clientBalance =
      await program.provider.connection.getTokenAccountBalance(clientUsdc)

    assert.ok(deal.offer.eq(offer))
    assert.ok(deal.deadline.eq(deadline))
    assert.ok(escrowBalance.value.uiAmountString === '103')
    assert.ok(clientBalance.value.uiAmountString === '897')
  })

  it('should allow the client to cancel a deal', async () => {
    await program.methods
      .closeDeal()
      .accounts({
        client: client.publicKey,
        deal: dealPda,
        escrow: dealEscrowPda,
        gig: gigPda,
        mint: usdc.token,
        ownerWallet: clientUsdc,
        signer: client.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([client])
      .rpc()

    const deal = await program.account.deal.fetchNullable(dealPda)

    const dealEscrow = await program.provider.connection.getAccountInfo(
      dealEscrowPda,
    )

    const clientBalance =
      await program.provider.connection.getTokenAccountBalance(clientUsdc)

    assert.ok(dealEscrow === null)
    assert.ok(deal === null)
    assert.ok(clientBalance.value.uiAmountString === '1000')
  })

  it('should allow the client to reopen the deal', async () => {
    await program.methods
      .createDeal({
        offer,
        deadline,
        referrer: null,
      })
      .accounts({
        config: configPda,
        deal: dealPda,
        gig: gigPda,
        mint: usdc.token,
        owner: client.publicKey,
        ownerWallet: clientUsdc,
        escrow: dealEscrowPda,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([client])
      .rpc()

    const deal = await program.account.deal.fetch(dealPda)

    const escrowBalance =
      await program.provider.connection.getTokenAccountBalance(dealEscrowPda)

    const clientBalance =
      await program.provider.connection.getTokenAccountBalance(clientUsdc)

    assert.ok(deal.offer.eq(offer))
    assert.ok(deal.deadline.eq(deadline))
    assert.ok(escrowBalance.value.uiAmountString === '103')
    assert.ok(clientBalance.value.uiAmountString === '897')
  })

  it('should allow the freelancer to accept a deal', async () => {
    await program.methods
      .acceptDeal()
      .accounts({
        deal: dealPda,
        freelancer: freelancer.publicKey,
        gig: gigPda,
      })
      .signers([freelancer])
      .rpc()

    const deal = await program.account.deal.fetch(dealPda)

    assert.ok((deal.state & 2) === 2)
  })

  it('should not allow the freelancer to close a gig with a pending deal', async () => {
    try {
      await program.methods
        .closeGig()
        .accounts({
          gig: gigPda,
          owner: freelancer.publicKey,
          payer: freelancer.publicKey,
          config: configPda,
        })
        .signers([freelancer])
        .rpc()

      assert.ok(false)
    } catch (e) {
      const err = e as AnchorError

      assert.ok(err.error.errorCode.code === 'ConstraintRaw')
    }
  })

  it('should not allow the client to close an accepted deal', async () => {
    try {
      await program.methods
        .closeDeal()
        .accounts({
          client: client.publicKey,
          deal: dealPda,
          escrow: dealEscrowPda,
          gig: gigPda,
          mint: usdc.token,
          ownerWallet: clientUsdc,
          signer: client.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([client])
        .rpc()

      assert.ok(false)
    } catch (e) {
      const err = e as AnchorError

      assert.ok(err.error.errorCode.code === 'ConstraintRaw')
    }
  })

  it('should allow the client to complete the deal', async () => {
    await program.methods
      .completeDeal()
      .accounts({
        client: client.publicKey,
        deal: dealPda,
        escrow: dealEscrowPda,
        gig: gigPda,
        mint: usdc.token,
        signer: client.publicKey,
        referrer: null,
        referrerTokenAccount: null,
        config: configPda,
        freelancer: freelancer.publicKey,
        freelancerTokenAccount: freelancerUsdc,
        treasury: treasury.publicKey,
        treasuryTokenAccount: treasuryUsdc,
        systemProgram: SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([client])
      .rpc()

    const freelancerBalance =
      await program.provider.connection.getTokenAccountBalance(freelancerUsdc)

    assert.ok(freelancerBalance.value.uiAmountString === '95')
  })

  it('should allow the freelancer to close a gig', async () => {
    await program.methods
      .closeGig()
      .accounts({
        gig: gigPda,
        owner: freelancer.publicKey,
        payer: freelancer.publicKey,
        config: configPda,
      })
      .signers([freelancer])
      .rpc()

    const gig = await program.account.gig.fetchNullable(gigPda)

    assert.ok(gig === null)
  })
})
