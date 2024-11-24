import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Soondex } from "../target/types/soondex";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { expect } from 'chai';
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getAssociatedTokenAddress,
  mintTo,
} from "@solana/spl-token";

describe("Soondex", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Soondex as Program<Soondex>;

  const wallet = provider.wallet as anchor.Wallet;
  let tokenXMint: PublicKey;
  let tokenYMint: PublicKey;
  let liquidityPoolPDA: PublicKey;
  let poolTokenXAccount: PublicKey;
  let poolTokenYAccount: PublicKey;

  before(async () => {
    const mintAuthority = Keypair.generate();
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

    [liquidityPoolPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), tokenXMint.toBuffer(), tokenYMint.toBuffer()],
      program.programId
    );

    poolTokenXAccount = await getAssociatedTokenAddress(
      tokenXMint,
      liquidityPoolPDA,
      true
    );
    poolTokenYAccount = await getAssociatedTokenAddress(
      tokenYMint,
      liquidityPoolPDA,
      true
    );
  });

  it("Initializes liquidity pool", async () => {
    const protocolWallet = Keypair.generate();

    await program.methods
      .initializePool(
        tokenXMint,
        tokenYMint,
        new anchor.BN(25),
        new anchor.BN(500)
      )
      .accounts({
        liquidityPool: liquidityPoolPDA, // Object literal may only specify known properties, and 'liquidityPool' does not exist in type 'ResolvedAccounts<{ name: "liquidityPool"; writable: true; pda: { seeds: [{ kind: "const"; value: [112, 111, 111, 108]; }, { kind: "arg"; path: "tokenXMint"; }, { kind: "arg"; path: "tokenYMint"; }]; }; } | { name: "authority"; writable: true; signer: true; }>
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
  });

  it("Places a buy order", async () => {
    const userTokenAccount = await getAssociatedTokenAddress(
      tokenXMint,
      wallet.publicKey
    );

    await program.methods
      .placeOrder(
        { buy: {} },
        new anchor.BN(1000),
        new anchor.BN(100)
      )
      .accounts({
        liquidityPool: liquidityPoolPDA,// Object literal may only specify known properties, and 'liquidityPool' does not exist in type 'ResolvedAccounts<{ name: "liquidityPool"; writable: true; pda: { seeds: [{ kind: "const"; value: [112, 111, 111, 108]; }, { kind: "arg"; path: "tokenXMint"; }, { kind: "arg"; path: "tokenYMint"; }]; }; } | { name: "authority"; writable: true; signer: true; }>
        user: wallet.publicKey,
        userTokenAccount,
        poolTokenAccount: poolTokenXAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
  });

  it("Cancels an order", async () => {
    await program.methods
      .cancelOrder(new anchor.BN(0))
      .accounts({
        liquidityPool: liquidityPoolPDA,// Object literal may only specify known properties, and 'liquidityPool' does not exist in type 'ResolvedAccounts<{ name: "liquidityPool"; writable: true; pda: { seeds: [{ kind: "const"; value: [112, 111, 111, 108]; }, { kind: "arg"; path: "tokenXMint"; }, { kind: "arg"; path: "tokenYMint"; }]; }; } | { name: "authority"; writable: true; signer: true; }>
        user: wallet.publicKey,
        userTokenAccount: poolTokenXAccount,
        poolTokenAccount: poolTokenXAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        tokenXMint,
        tokenYMint,
      })
      .rpc();
  });

  it("Swaps tokens", async () => {
    const userTokenXAccount = await getAssociatedTokenAddress(
      tokenXMint,
      wallet.publicKey
    );
    const userTokenYAccount = await getAssociatedTokenAddress(
      tokenYMint,
      wallet.publicKey
    );

    await mintTo(
      provider.connection,
      wallet.payer,
      tokenXMint,
      userTokenXAccount,
      wallet.publicKey,
      1000000000
    );

    const initialUserXBalance = await provider.connection.getTokenAccountBalance(userTokenXAccount);
    const initialUserYBalance = await provider.connection.getTokenAccountBalance(userTokenYAccount);

    await program.methods
      .swapTokens(
        new anchor.BN(1000000),
        new anchor.BN(900000)
      )
      .accounts({
        liquidityPool: liquidityPoolPDA,// Object literal may only specify known properties, and 'liquidityPool' does not exist in type 'ResolvedAccounts<{ name: "liquidityPool"; writable: true; pda: { seeds: [{ kind: "const"; value: [112, 111, 111, 108]; }, { kind: "arg"; path: "tokenXMint"; }, { kind: "arg"; path: "tokenYMint"; }]; }; } | { name: "authority"; writable: true; signer: true; }>
        user: wallet.publicKey,
        userTokenIn: userTokenXAccount,
        userTokenOut: userTokenYAccount,
        poolTokenIn: poolTokenXAccount,
        poolTokenOut: poolTokenYAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        tokenXMint,
        tokenYMint
      })
      .rpc();

    const finalUserXBalance = await provider.connection.getTokenAccountBalance(userTokenXAccount);
    const finalUserYBalance = await provider.connection.getTokenAccountBalance(userTokenYAccount);
    const poolAccount = await program.account.liquidityPool.fetch(liquidityPoolPDA);

    expect(poolAccount.volume24h.toNumber()).to.be.above(0);
    expect(poolAccount.fees24h.toNumber()).to.be.above(0);
    expect(Number(finalUserXBalance.value.amount)).to.be.below(Number(initialUserXBalance.value.amount));
    expect(Number(finalUserYBalance.value.amount)).to.be.above(Number(initialUserYBalance.value.amount));
  });

  it("Stakes tokens", async () => {
    const userTokenAccount = await getAssociatedTokenAddress(
      tokenXMint,
      wallet.publicKey
    );

    const userState = Keypair.generate();

    await program.methods
      .stakeTokens(new anchor.BN(100000))
      .accounts({
        liquidityPool: liquidityPoolPDA,// Object literal may only specify known properties, and 'liquidityPool' does not exist in type 'ResolvedAccounts<{ name: "liquidityPool"; writable: true; pda: { seeds: [{ kind: "const"; value: [112, 111, 111, 108]; }, { kind: "arg"; path: "tokenXMint"; }, { kind: "arg"; path: "tokenYMint"; }]; }; } | { name: "authority"; writable: true; signer: true; }>
        user: wallet.publicKey,
        userState: userState.publicKey,
        userTokenAccount,
        poolTokenAccount: poolTokenXAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([userState])
      .rpc();

    const stateAccount = await program.account.userState.fetch(userState.publicKey);
    expect(stateAccount.amountStaked.toNumber()).to.equal(100000);
  });

  it("Withdraws tokens", async () => {
    const userTokenAccount = await getAssociatedTokenAddress(
      tokenXMint,
      wallet.publicKey
    );

    const userState = Keypair.generate();

    await program.methods
      .withdrawTokens()
      .accounts({
        liquidityPool: liquidityPoolPDA,// Object literal may only specify known properties, and 'liquidityPool' does not exist in type 'ResolvedAccounts<{ name: "liquidityPool"; writable: true; pda: { seeds: [{ kind: "const"; value: [112, 111, 111, 108]; }, { kind: "arg"; path: "tokenXMint"; }, { kind: "arg"; path: "tokenYMint"; }]; }; } | { name: "authority"; writable: true; signer: true; }>
        user: wallet.publicKey,
        userState: userState.publicKey,
        userTokenAccount,
        poolTokenAccount: poolTokenXAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([userState])
      .rpc();

    const stateAccount = await program.account.userState.fetch(userState.publicKey);
    expect(stateAccount.amountStaked.toNumber()).to.equal(0);
  });

  it("Matches orders", async () => {
    await program.methods
      .matchOrders()
      .accounts({
        liquidityPool: liquidityPoolPDA,// Object literal may only specify known properties, and 'liquidityPool' does not exist in type 'ResolvedAccounts<{ name: "liquidityPool"; writable: true; pda: { seeds: [{ kind: "const"; value: [112, 111, 111, 108]; }, { kind: "arg"; path: "tokenXMint"; }, { kind: "arg"; path: "tokenYMint"; }]; }; } | { name: "authority"; writable: true; signer: true; }>
        authority: wallet.publicKey,
      })
      .rpc();

    const poolAccount = await program.account.liquidityPool.fetch(liquidityPoolPDA);
    expect(poolAccount.orders.length).to.equal(0);
  });});
