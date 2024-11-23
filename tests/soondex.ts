import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Soondex } from "../target/types/soondex";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount } from "@solana/spl-token";
import { assert } from "chai";

describe("soondex", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Soondex as Program<Soondex>;
  
  const payer = Keypair.generate();
  const tokenXMint = Keypair.generate();
  const tokenYMint = Keypair.generate();
  let poolTokenXAccount: PublicKey;
  let poolTokenYAccount: PublicKey;
  let protocolWallet: Keypair;

  const MAX_FEE_RATE = new BN(10000);
  const MAX_REWARD_RATE = new BN(1000);
  
  before(async () => {
    const signature = await provider.connection.requestAirdrop(
      payer.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(signature);

    await createMint(
      provider.connection,
      payer,
      payer.publicKey,
      null,
      9,
      tokenXMint
    );
    
    await createMint(
      provider.connection,
      payer,
      payer.publicKey,
      null,
      9,
      tokenYMint
    );

    protocolWallet = Keypair.generate();
  });

  it("Initialize Pool", async () => {
    const [poolPda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool")],
      program.programId
    );

    poolTokenXAccount = await createAccount(
      provider.connection,
      payer,
      tokenXMint.publicKey,
      poolPda
    );

    poolTokenYAccount = await createAccount(
      provider.connection,
      payer,
      tokenYMint.publicKey,
      poolPda
    );

    const feeRate = new BN(25);
    const rewardRate = new BN(100);

    try {
      await program.methods
        .initializePool(
          {
            authority: payer.publicKey,
            tokenXReserve: new BN(0),
            tokenYReserve: new BN(0),
            lpTokenSupply: new BN(0),
            lpTokens: [],
            feeRate: new BN(feeRate),
            rewardRate: new BN(rewardRate),
            totalStaked: new BN(0),
            bump,
            orderCount: new BN(0),
            orders: [],
            volume24h: new BN(0),
            fees24h: new BN(0),
            lastVolumeReset: new BN(0),
            tvlX: new BN(0),
            tvlY: new BN(0),
            stakingRewards: new BN(0),
          },
          tokenXMint.publicKey,
          tokenYMint.publicKey,
          feeRate,
          rewardRate
        )
        .accounts({
          poolAccount: poolPda,
          payer: payer.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
          tokenXMint: tokenXMint.publicKey,
          tokenYMint: tokenYMint.publicKey,
          poolTokenXAccount,
          poolTokenYAccount,
          protocolWallet: protocolWallet.publicKey,
        })
        .signers([payer])
        .rpc();

      const poolAccount = await program.account.liquidityPool.fetch(poolPda);
      assert.equal(poolAccount.authority.toBase58(), payer.publicKey.toBase58());
      assert.equal(poolAccount.feeRate.toNumber(), feeRate.toNumber());
      assert.equal(poolAccount.rewardRate.toNumber(), rewardRate.toNumber());
      assert.equal(poolAccount.tokenXReserve.toNumber(), 0);
      assert.equal(poolAccount.tokenYReserve.toNumber(), 0);
      assert.equal(poolAccount.lpTokenSupply.toNumber(), 0);
      
    } catch (error) {
      console.error("Error:", error);
      throw error;
    }
  });
});
