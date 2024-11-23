import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Soondex } from "../target/types/soondex";
import { startAnchor } from "solana-bankrun";
import { PublicKey, Keypair } from "@solana/web3.js";
import { expect } from "chai";
import { BankrunProvider } from "anchor-bankrun";

const IDL = require("../target/idl/soondex.json");
const PROGRAM_ID = new PublicKey("B4xt3vAan4S5UmUgucsxMPi2uwqEmrSSdvJnzVPWeUFu");

describe("Soondex Program Tests", function () {
  let soondexProgram: Program<Soondex>;
  let wallet: Keypair;
  let tokenXMint: Keypair;
  let tokenYMint: Keypair;
  let poolPDA: PublicKey;
  let context: any;

  before(async () => {
    context = await startAnchor("", [{ name: 'soondex', programId: PROGRAM_ID }], []);
    const provider = new BankrunProvider(context);
    soondexProgram = new Program<Soondex>(IDL, provider);

    wallet = Keypair.generate();
    tokenXMint = Keypair.generate();
    tokenYMint = Keypair.generate();

    [poolPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool")],
      PROGRAM_ID
    );

    const liquidityPoolData = {
      authority: wallet.publicKey,
      tokenXReserve: new BN(0),
      tokenYReserve: new BN(0),
      lpTokenSupply: new BN(0),
      lpTokens: [],
      feeRate: new BN(25),
      rewardRate: new BN(100),
      totalStaked: new BN(0),
      bump: 0,
      orderCount: new BN(0),
      orders: [],
      volume24h: new BN(0),
      fees24h: new BN(0),
      lastVolumeReset: new BN(0),
      tvlX: new BN(0),
      tvlY: new BN(0),
      stakingRewards: new BN(0)
    };

    await soondexProgram.methods
      .initializePool(
        liquidityPoolData,
        tokenXMint.publicKey,
        tokenYMint.publicKey,
        new BN(25),
        new BN(100)
      )
      .accounts({
        liquidityPool: poolPDA, //Having trouble liquiditypool doesnt exiat in type resoved accounts
        payer: wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        tokenXMint: tokenXMint.publicKey,
        tokenYMint: tokenYMint.publicKey,
        protocolWallet: wallet.publicKey
      })
      .signers([wallet])
      .rpc();
  });

  it("should initialize the liquidity pool correctly", async () => {
    const poolState = await soondexProgram.account.liquidityPool.fetch(poolPDA);
    
    expect(poolState.feeRate.toNumber()).to.equal(25);
    expect(poolState.rewardRate.toNumber()).to.equal(100);
    expect(poolState.tokenXReserve.toNumber()).to.equal(0);
    expect(poolState.tokenYReserve.toNumber()).to.equal(0);
  });

  after(async () => {
    await context.close();
  });
});