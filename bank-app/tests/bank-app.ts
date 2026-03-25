import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BankApp } from "../target/types/bank_app";
import { StakingApp } from "../target/types/staking_app";
import { PublicKey, SystemProgram, TransactionInstruction } from "@solana/web3.js";
import { BN } from "bn.js";
import { 
  createAssociatedTokenAccountInstruction, 
  getAssociatedTokenAddressSync, 
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID, 
  createMint,                  
  getOrCreateAssociatedTokenAccount,
  mintTo
} from "@solana/spl-token";

describe("staking-test", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.BankApp as Program<BankApp>;
  const stakingProgram = anchor.workspace.StakingApp as Program<StakingApp>;

  const BANK_APP_ACCOUNTS = {
    bankInfo: PublicKey.findProgramAddressSync([Buffer.from("BANK_INFO_SEED")], program.programId)[0],
    bankVault: PublicKey.findProgramAddressSync([Buffer.from("BANK_VAULT_SEED")], program.programId)[0],
  };

  const [stakingVault] = PublicKey.findProgramAddressSync([Buffer.from("STAKING_VAULT")], stakingProgram.programId);
  const [globalVault] = PublicKey.findProgramAddressSync([Buffer.from("GLOBAL_STATE")], stakingProgram.programId);

  let mint: PublicKey;
  let userAta: PublicKey;


  it("Initialize Bank and Global Vault", async () => {
    try {
      await program.methods.initialize().accounts({
        bankInfo: BANK_APP_ACCOUNTS.bankInfo,
        bankVault: BANK_APP_ACCOUNTS.bankVault,
        authority: provider.publicKey,
        systemProgram: SystemProgram.programId
      }).rpc();
    } catch (e) {}

    try {
      const [tempUserInfo] = PublicKey.findProgramAddressSync(
        [Buffer.from("USER_INFO"), provider.publicKey.toBuffer()], 
        stakingProgram.programId
      );
      await stakingProgram.methods.stake(new BN(0), true).accounts({
        stakingVault: stakingVault,
        globalVault: globalVault,
        userInfo: tempUserInfo,
        user: provider.publicKey,
        payer: provider.publicKey,
        systemProgram: SystemProgram.programId,
      }).rpc();
    } catch (e) {}
  });

  it("Deposit SOL to Bank", async () => {
    const [userReserve] = PublicKey.findProgramAddressSync(
        [Buffer.from("USER_RESERVE_SEED"), provider.publicKey.toBuffer()], 
        program.programId
    );
    await program.methods.deposit(new BN(5_000_000))
      .accounts({
        bankInfo: BANK_APP_ACCOUNTS.bankInfo,
        bankVault: BANK_APP_ACCOUNTS.bankVault,
        userReserve: userReserve,
        user: provider.publicKey,
        systemProgram: SystemProgram.programId,
      }).rpc();
  });

  it("Stake SOL", async () => {
    const [stakingUserInfo] = PublicKey.findProgramAddressSync(
      [Buffer.from("USER_INFO"), BANK_APP_ACCOUNTS.bankVault.toBuffer()], 
      stakingProgram.programId
    );

    await program.methods.invest(new BN(1_000_000), true)
      .accounts({
        bankInfo: BANK_APP_ACCOUNTS.bankInfo,
        bankVault: BANK_APP_ACCOUNTS.bankVault,
        stakingVault: stakingVault,
        stakingGlobalVault: globalVault, 
        stakingInfo: stakingUserInfo,
        stakingProgram: stakingProgram.programId,
        authority: provider.publicKey,
        systemProgram: SystemProgram.programId
      }).rpc();
  });

  it("Add Rewards to Pool", async () => {
    await stakingProgram.methods.addRewards(new BN(500_000)) 
      .accounts({
        globalVault: globalVault,
        stakingVault: stakingVault,
        user: provider.publicKey,
        systemProgram: SystemProgram.programId,
      }).rpc();
  });

  it("Unstake SOL (Shares to SOL)", async () => {
    const [stakingUserInfo] = PublicKey.findProgramAddressSync(
      [Buffer.from("USER_INFO"), BANK_APP_ACCOUNTS.bankVault.toBuffer()], 
      stakingProgram.programId
    );

    await program.methods.invest(new BN(500_000), false)
      .accounts({
        bankInfo: BANK_APP_ACCOUNTS.bankInfo,
        bankVault: BANK_APP_ACCOUNTS.bankVault,
        stakingVault: stakingVault,
        stakingGlobalVault: globalVault, 
        stakingInfo: stakingUserInfo,
        stakingProgram: stakingProgram.programId,
        authority: provider.publicKey,
        systemProgram: SystemProgram.programId
      }).rpc();
  });

});