import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Soondex } from "../target/types/soondex";
import { assert } from "chai";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  getAssociatedTokenAddress,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { Keypair, SystemProgram, PublicKey } from "@solana/web3.js";

describe("Soondex DEX", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Soondex as Program<Soondex>;
  const wallet = provider.wallet as anchor.Wallet;

  const getUserStateAddress = async (liquidityPool: PublicKey): Promise<PublicKey> => {
    const [userStatePDA] = await PublicKey.findProgramAddress(
        [
            Buffer.from("user_state"),
            liquidityPool.toBuffer(),
            wallet.publicKey.toBuffer()
        ],
        program.programId
    );
    return userStatePDA;
};

  let tokenXMint: PublicKey;
  let tokenYMint: PublicKey;
  let liquidityPoolPDA: PublicKey;
  let poolTokenXAccount: PublicKey;
  let poolTokenYAccount: PublicKey;
  let userTokenXAccount: PublicKey;
  let userTokenYAccount: PublicKey;
  let mintAuthority: Keypair;
  let protocolWallet: Keypair;
  let adminKeypair: Keypair;
  let bump: number;

  before(async () => {
    console.log("\n=== Setting up Test Environment ===");

    mintAuthority = Keypair.generate();
    protocolWallet = Keypair.generate();
    adminKeypair = Keypair.generate();
    
    console.log("Generated Keys:");
    console.log(`Mint Authority: ${mintAuthority.publicKey}`);
    console.log(`Protocol Wallet: ${protocolWallet.publicKey}`);
    console.log(`Admin: ${adminKeypair.publicKey}`);

    const airdropTx = await provider.connection.requestAirdrop(
      mintAuthority.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropTx);
    console.log("✓ Airdropped SOL to mint authority");

    tokenXMint = await createMint(
      provider.connection,
      mintAuthority,
      mintAuthority.publicKey,
      null,
      9
    );
    tokenYMint = await createMint(
      provider.connection,
      mintAuthority,
      mintAuthority.publicKey,
      null,
      9
    );
    console.log("✓ Created token mints:", {
      tokenX: tokenXMint.toString(),
      tokenY: tokenYMint.toString()
    });

    [liquidityPoolPDA, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), tokenXMint.toBuffer(), tokenYMint.toBuffer()],
      program.programId
    );
    console.log("✓ Generated liquidity pool PDA:", liquidityPoolPDA.toString());

    poolTokenXAccount = await getAssociatedTokenAddress(tokenXMint, liquidityPoolPDA, true);
    poolTokenYAccount = await getAssociatedTokenAddress(tokenYMint, liquidityPoolPDA, true);
    
    userTokenXAccount = await createAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      tokenXMint,
      wallet.publicKey
    );
    userTokenYAccount = await createAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      tokenYMint,
      wallet.publicKey
    );
    console.log("✓ Created token accounts");

    await createAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      tokenXMint,
      liquidityPoolPDA,
      undefined,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
      true
    );
    await createAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      tokenYMint,
      liquidityPoolPDA,
      undefined,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
      true
    );
    console.log("✓ Initialized pool token accounts");

    await mintTo(
      provider.connection,
      mintAuthority,
      tokenXMint,
      userTokenXAccount,
      mintAuthority.publicKey,
      1_000_000_000
    );
    await mintTo(
      provider.connection,
      mintAuthority,
      tokenYMint,
      userTokenYAccount,
      mintAuthority.publicKey,
      1_000_000_000
    );
    console.log("✓ Minted initial tokens to user");

    await program.methods
      .initializePool(
        tokenXMint,
        tokenYMint,
        new anchor.BN(25),

      )
      .accountsStrict({
        liquidityPool: liquidityPoolPDA,
        payer: wallet.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenXMint,
        tokenYMint,
        poolTokenXAccount,
        poolTokenYAccount,
        protocolWallet: protocolWallet.publicKey,
      })
      .rpc();
    console.log("✓ Initialized liquidity pool");

    await program.methods
      .manageAdmin(adminKeypair.publicKey, true)
      .accountsStrict({
        liquidityPool: liquidityPoolPDA,
        authority: wallet.publicKey,
        tokenXMint,
        tokenYMint,
      })
      .rpc();
    console.log("✓ Setup admin");
  });

  it("Add Initial Liquidity", async () => {
    console.log("\n=== Adding Initial Liquidity ==="); // 2% slippage tolerance
    const amountX = 100_000_000;
    const amountY = 100_000_000;
    const params = {
        amountX: new anchor.BN(100_000_000),
        amountY: new anchor.BN(100_000_000),
        minLpTokens: new anchor.BN(1000)
    };
    
    const tx = await program.methods
      .addLiquidity(
        tokenXMint,
        tokenYMint,
        params.amountX,
        params.amountY
      )
      .accountsStrict({
        tokenXMint,
        tokenYMint,
        liquidityPool: liquidityPoolPDA,
        user: wallet.publicKey,
        userTokenXAccount: userTokenXAccount,
        userTokenYAccount: userTokenYAccount,
        poolTokenXAccount: poolTokenXAccount,
        poolTokenYAccount: poolTokenYAccount,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .rpc();
    
    console.log("Add Liquidity TX:", tx);
    console.log("✓ Initial liquidity added successfully");
});

  it("Swap Tokens", async () => {
    console.log("\n=== Performing Token Swap ===");
    
    const params = {
        inputToken: tokenXMint,
        outputToken: tokenYMint,
        amountIn: new anchor.BN(10_000_000),
        minimumAmountOut: new anchor.BN(9_000_000)
    };
    
    const tx = await program.methods
      .swapTokens(
        params.inputToken,
        params.outputToken,
        params.amountIn,
        params.minimumAmountOut
      )
      .accountsStrict({
        liquidityPool: liquidityPoolPDA,
        user: wallet.publicKey,
        userTokenIn: userTokenXAccount,
        userTokenOut: userTokenYAccount,
        poolTokenX: poolTokenXAccount,
        poolTokenY: poolTokenYAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        tokenXMint,
        tokenYMint
      })
      .rpc();
    
    console.log("Swap TX:", tx);
    console.log("✓ Swap executed successfully");
});

it("Remove Liquidity", async () => {
  console.log("\n=== Removing Liquidity ===");
  const amount = new anchor.BN(50_000_000); // Remove half of initial liquidity
  
  const tx = await program.methods
    .removeLiquidity(
      tokenXMint,
      tokenYMint,
      amount,
      amount
    )
    .accountsStrict({
      liquidityPool: liquidityPoolPDA,
      user: wallet.publicKey,
      userTokenXAccount,
      userTokenYAccount,
      poolTokenXAccount,
      poolTokenYAccount,
      tokenProgram: TOKEN_PROGRAM_ID,
      tokenXMint,
      tokenYMint
    })
    .rpc();
  
  console.log("Remove Liquidity TX:", tx);
});

it("Stake Tokens", async () => {
  console.log("\n=== Staking Tokens ===");
  const stakeAmount = new anchor.BN(20_000_000);
  
  const userStatePDA = await getUserStateAddress(liquidityPoolPDA);
  
  const tx = await program.methods
    .stake(stakeAmount)
    .accountsStrict({
      liquidityPool: liquidityPoolPDA,
      userState: userStatePDA,
      user: wallet.publicKey,
      userTokenAccount: userTokenXAccount,
      poolTokenAccount: poolTokenXAccount,
      tokenMint: tokenXMint,
      tokenXMint,
      tokenYMint,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId
    })
    .rpc();
  
  console.log("Stake TX:", tx);
});

it("Unstake Tokens", async () => {
  console.log("\n=== Unstaking Tokens ===");
  const unstakeAmount = new anchor.BN(10_000_000);

  const userStatePDA = await getUserStateAddress(liquidityPoolPDA);

  const tx = await program.methods
    .unstake(unstakeAmount)
    .accountsStrict({
      liquidityPool: liquidityPoolPDA,
      userState: userStatePDA,
      user: wallet.publicKey,
      userTokenAccount: userTokenXAccount,
      poolTokenAccount: poolTokenXAccount,
      tokenMint: tokenXMint,
      tokenXMint,
      tokenYMint,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId
    })
    .rpc();

  console.log("Unstake TX:", tx);
});
it("Edge Case: Attempt max possible swap", async () => {
  console.log("\n=== Testing Edge Case: Max Swap ===");
  const maxAmount = new anchor.BN(1_000_000_000);
  
  try {
      await program.methods
        .swapTokens(
          tokenXMint,
          tokenYMint,
          maxAmount,
          new anchor.BN(0)
        )
        .accountsStrict({
          liquidityPool: liquidityPoolPDA,
          user: wallet.publicKey,
          userTokenIn: userTokenXAccount,
          userTokenOut: userTokenYAccount,
          poolTokenX: poolTokenXAccount,
          poolTokenY: poolTokenYAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          tokenXMint,
          tokenYMint
        })
        .rpc();
      assert(false, "Expected transaction to fail");
  } catch (e) {
      assert(e.message.includes("InvalidSwapInput") || e.message.includes("InsufficientFunds"));
  }
});

it("Multiple Operations Sequence", async () => {
  console.log("\n=== Testing Operation Sequence ===");
  
  const poolAccount = await program.account.liquidityPool.fetch(liquidityPoolPDA);
  
  // Use exact pool ratios
  const amount = new anchor.BN(10_000_000);
  const amountY = amount
      .mul(new anchor.BN(poolAccount.tokenYReserve))
      .div(new anchor.BN(poolAccount.tokenXReserve));
  
  await program.methods
    .addLiquidity(
      tokenXMint,
      tokenYMint,
      amount,
      amountY
    )
    .accountsStrict({
      tokenXMint,
      tokenYMint,
      liquidityPool: liquidityPoolPDA,
      user: wallet.publicKey,
      userTokenXAccount,
      userTokenYAccount,
      poolTokenXAccount,
      poolTokenYAccount,
      tokenProgram: TOKEN_PROGRAM_ID
    })
    .rpc();
  
  console.log("✓ Sequential operations completed successfully");
});


it("Admin Management", async () => {
  console.log("\n=== Testing Admin Management ===");
  
  const newAdminKeypair = Keypair.generate();
  console.log("Generated new admin:", newAdminKeypair.publicKey.toString());

  // Add new admin
  await program.methods
    .manageAdmin(newAdminKeypair.publicKey, true)
    .accountsStrict({
      liquidityPool: liquidityPoolPDA,
      authority: wallet.publicKey,
      tokenXMint,
      tokenYMint,
    })
    .rpc();
  console.log("✓ Added new admin successfully");

  // Verify admin was added
  const poolAfterAdd = await program.account.liquidityPool.fetch(liquidityPoolPDA);
  assert(poolAfterAdd.admins.some(admin => admin.equals(newAdminKeypair.publicKey)), 
    "New admin should be in the admins list");

  // Remove admin
  await program.methods
    .manageAdmin(newAdminKeypair.publicKey, false)
    .accountsStrict({
      liquidityPool: liquidityPoolPDA,
      authority: wallet.publicKey,
      tokenXMint,
      tokenYMint,
    })
    .rpc();
  console.log("✓ Removed admin successfully");

  // Verify admin was removed
  const poolAfterRemove = await program.account.liquidityPool.fetch(liquidityPoolPDA);
  assert(!poolAfterRemove.admins.some(admin => admin.equals(newAdminKeypair.publicKey)), 
    "Removed admin should not be in the admins list");
});


});