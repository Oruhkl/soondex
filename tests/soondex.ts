import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Soondex } from "../target/types/soondex";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getAssociatedTokenAddress,
  createAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";

describe("Soondex", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Soondex as Program<Soondex>;

  const wallet = provider.wallet as anchor.Wallet;
  let tokenXMint: PublicKey;
  let tokenYMint: PublicKey;
  let poolAccount: PublicKey;
  let userTokenAccount: PublicKey;

  before(async () => {
    tokenXMint = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      null,
      9
    );

    tokenYMint = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      null,
      9
    );

    const [pool] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), tokenXMint.toBuffer(), tokenYMint.toBuffer()],
      program.programId
    );
    poolAccount = pool;
  });

  it("Initializes pool", async () => {
    const protocolWallet = Keypair.generate();
    const poolTokenXAccount = await getAssociatedTokenAddress(
      tokenXMint,
      poolAccount,
      true
    );
    const poolTokenYAccount = await getAssociatedTokenAddress(
      tokenYMint,
      poolAccount,
      true
    );

    await program.methods
      .initializePool(
        tokenXMint,
        tokenYMint,
        new anchor.BN(25),
        new anchor.BN(100)
      )
      .accounts({
        pool: poolAccount,
        payer: wallet.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenXMint,
        tokenYMint,
        poolTokenXAccount,
        poolTokenYAccount,
        protocolWallet: protocolWallet.publicKey
      })
      .rpc();
  });

  it("Stakes tokens", async () => {
    const userState = Keypair.generate();
    const poolTokenAccount = await getAssociatedTokenAddress(
      tokenXMint,
      poolAccount,
      true
    );

    await program.methods
      .stakeTokens(new anchor.BN(500000))
      .accounts({
        pool: poolAccount,
        user: wallet.publicKey,
        userState: userState.publicKey,
        userTokenAccount,
        poolTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([userState])
      .rpc();
  });

  it("Swaps tokens", async () => {
    const userTokenIn = await getAssociatedTokenAddress(
      tokenXMint,
      wallet.publicKey
    );
    const userTokenOut = await getAssociatedTokenAddress(
      tokenYMint,
      wallet.publicKey
    );
    const poolTokenIn = await getAssociatedTokenAddress(
      tokenXMint,
      poolAccount,
      true
    );
    const poolTokenOut = await getAssociatedTokenAddress(
      tokenYMint,
      poolAccount,
      true
    );

    await program.methods
      .swapTokens(new anchor.BN(100000), new anchor.BN(90000))
      .accounts({
        pool: poolAccount,
        user: wallet.publicKey,
        userTokenIn,
        userTokenOut,
        poolTokenIn,
        poolTokenOut,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .rpc();
  });
});
