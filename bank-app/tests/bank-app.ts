import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BankApp } from "../target/types/bank_app";
import { StakingApp } from "../target/types/staking_app";
import { PublicKey, SystemProgram, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { BN } from "bn.js";
import { assert } from "chai";

describe("Dự án Invest SOL & Staking SOL", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const bankProgram = anchor.workspace.BankApp as Program<BankApp>;
  const stakingProgram = anchor.workspace.StakingApp as Program<StakingApp>;

  const userB = Keypair.generate();

  const BANK_INFO_SEED = Buffer.from("BANK_INFO_SEED");
  const BANK_VAULT_SEED = Buffer.from("BANK_VAULT_SEED");
  const STAKING_VAULT_SEED = Buffer.from("STAKING_VAULT");
  const GLOBAL_STATE_SEED = Buffer.from("GLOBAL_STATE");
  const USER_INFO_SEED = Buffer.from("USER_INFO");
  const USER_RESERVE_SEED = Buffer.from("USER_RESERVE_SEED");

  const [bankInfo] = PublicKey.findProgramAddressSync([BANK_INFO_SEED], bankProgram.programId);
  const [bankVault] = PublicKey.findProgramAddressSync([BANK_VAULT_SEED], bankProgram.programId);
  const [stakingVault] = PublicKey.findProgramAddressSync([STAKING_VAULT_SEED], stakingProgram.programId);
  const [globalVault] = PublicKey.findProgramAddressSync([GLOBAL_STATE_SEED], stakingProgram.programId);

  const getStakingInfoPda = (owner: PublicKey) => 
    PublicKey.findProgramAddressSync([USER_INFO_SEED, owner.toBuffer()], stakingProgram.programId)[0];

  it("1. Setup hệ thống & Airdrop cho User B", async () => {
    const transferTx = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: provider.publicKey, 
        toPubkey: userB.publicKey,      
        lamports: 2 * LAMPORTS_PER_SOL, 
      })
    );
    await provider.sendAndConfirm(transferTx);
    console.log("User B đã có 2 SOL.");
    try {
      await bankProgram.account.bankInfo.fetch(bankInfo);
    } catch (err) {
      console.log("⏳ Đang khởi tạo Bank Program...");
      await bankProgram.methods.initialize().accounts({
        bankInfo, 
        bankVault, 
        authority: provider.publicKey, 
        systemProgram: SystemProgram.programId
      }).rpc();
    }
    try {
      await stakingProgram.account.globalVault.fetch(globalVault);
    } catch (err) {
      console.log("Đang khởi tạo Staking Program...");
      await stakingProgram.methods.initialize().accounts({
        globalVault, 
        admin: provider.publicKey, 
        systemProgram: SystemProgram.programId,
      }).rpc();
    }
  });

  it("2. User A: Nạp 1 SOL -> Đầu tư 1 SOL (CPI)", async () => {
    const userReserveA = PublicKey.findProgramAddressSync([USER_RESERVE_SEED, provider.publicKey.toBuffer()], bankProgram.programId)[0];
    const stakingInfoForBank = getStakingInfoPda(bankVault);

    await bankProgram.methods.deposit(new BN(1 * LAMPORTS_PER_SOL)).accounts({
      bankInfo, bankVault, userReserve: userReserveA, user: provider.publicKey, systemProgram: SystemProgram.programId,
    }).rpc();

    await bankProgram.methods.invest(new BN(1 * LAMPORTS_PER_SOL), true).accounts({
      bankInfo, bankVault, stakingVault, stakingGlobalVault: globalVault,
      stakingInfo: stakingInfoForBank, stakingProgram: stakingProgram.programId,
      authority: provider.publicKey, systemProgram: SystemProgram.programId
    }).rpc();

    const vault = await stakingProgram.account.globalVault.fetch(globalVault);
    console.log(`Pool hiện có: ${vault.totalAssets.toNumber() / LAMPORTS_PER_SOL} SOL.`);
    assert.equal(vault.totalAssets.toNumber(), 1 * LAMPORTS_PER_SOL);
  });

  it("3. Admin bơm lãi 0.5 SOL", async () => {
    await stakingProgram.methods.addRewards(new BN(0.5 * LAMPORTS_PER_SOL)).accounts({
      globalVault, stakingVault, admin: provider.publicKey, systemProgram: SystemProgram.programId,
    }).rpc();
    console.log("Đã bơm lãi. Tổng tài sản thực tế: 1.5 SOL.");
  });

  it("4. User B: Nạp 1 SOL", async () => {
    const infoB = getStakingInfoPda(userB.publicKey);
    await stakingProgram.methods.stake(new BN(1 * LAMPORTS_PER_SOL), true).accounts({
      stakingVault, globalVault, userInfo: infoB, user: userB.publicKey, payer: userB.publicKey, systemProgram: SystemProgram.programId,
    }).signers([userB]).rpc();

    const userStateB = await stakingProgram.account.userInfo.fetch(infoB);
    console.log(`User B nhận về: ${userStateB.shares.toNumber() / LAMPORTS_PER_SOL} Shares.`);
    assert.isBelow(userStateB.shares.toNumber(), 1 * LAMPORTS_PER_SOL);
  });

  it("5. Kết thúc: Cả hai cùng rút tiền", async () => {
    const stakingInfoForBank = getStakingInfoPda(bankVault);
    const infoB = getStakingInfoPda(userB.publicKey);

    const accountA = await stakingProgram.account.userInfo.fetch(stakingInfoForBank);
    const accountB = await stakingProgram.account.userInfo.fetch(infoB);

    await Promise.all([
      bankProgram.methods.invest(accountA.shares, false).accounts({
        bankInfo, bankVault, stakingVault, stakingGlobalVault: globalVault,
        stakingInfo: stakingInfoForBank, stakingProgram: stakingProgram.programId,
        authority: provider.publicKey, systemProgram: SystemProgram.programId
      }).rpc(),
      stakingProgram.methods.stake(accountB.shares, false).accounts({
        stakingVault, globalVault, userInfo: infoB, user: userB.publicKey, payer: userB.publicKey, systemProgram: SystemProgram.programId,
      }).signers([userB]).rpc()
    ]);

    const finalVault = await stakingProgram.account.globalVault.fetch(globalVault);
    console.log(`Test thành công! Pool còn: ${finalVault.totalAssets.toNumber()} Lamports.`);
    assert.equal(finalVault.totalShares.toNumber(), 0);
  });
});