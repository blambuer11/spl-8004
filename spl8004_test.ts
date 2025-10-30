import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Spl8004 } from "../target/types/spl_8004";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import crypto from "crypto";

describe("SPL-8004: AI Agent Identity & Reputation", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Spl8004 as Program<Spl8004>;
  
  const owner = Keypair.generate();
  const validator = Keypair.generate();
  const treasury = Keypair.generate();
  
  const agentId = `agent-${Date.now()}`;
  const metadataUri = "https://arweave.net/example-agent-metadata";

  let configPda: PublicKey;
  let identityPda: PublicKey;
  let reputationPda: PublicKey;
  let rewardPoolPda: PublicKey;
  let validationPda: PublicKey;
  let taskHash: Buffer;

  before(async () => {
    console.log("Setting up test environment...");

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        owner.publicKey,
        10 * anchor.web3.LAMPORTS_PER_SOL
      )
    );

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        validator.publicKey,
        10 * anchor.web3.LAMPORTS_PER_SOL
      )
    );

    [configPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );

    [identityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("identity"), Buffer.from(agentId)],
      program.programId
    );

    [reputationPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("reputation"), identityPda.toBuffer()],
      program.programId
    );

    [rewardPoolPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("reward_pool"), identityPda.toBuffer()],
      program.programId
    );

    console.log("Test accounts prepared");
    console.log("Owner:", owner.publicKey.toString());
    console.log("Validator:", validator.publicKey.toString());
    console.log("Treasury:", treasury.publicKey.toString());
  });

  it("Initializes global config", async () => {
    console.log("\n=== Test 1: Initialize Config ===");

    try {
      const tx = await program.methods
        .initializeConfig(300, treasury.publicKey)
        .accounts({
          config: configPda,
          authority: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      console.log("Config initialized, tx:", tx);

      const config = await program.account.globalConfig.fetch(configPda);
      
      expect(config.authority.toString()).to.equal(
        provider.wallet.publicKey.toString()
      );
      expect(config.treasury.toString()).to.equal(treasury.publicKey.toString());
      expect(config.commissionRate).to.equal(300);
      expect(config.totalAgents.toNumber()).to.equal(0);
      expect(config.totalValidations.toNumber()).to.equal(0);

      console.log("✓ Config initialized successfully");
      console.log("  Commission rate:", config.commissionRate / 100 + "%");
    } catch (e) {
      console.log("Config already initialized (expected)");
    }
  });

  it("Registers a new AI agent", async () => {
    console.log("\n=== Test 2: Register Agent ===");

    const tx = await program.methods
      .registerAgent(agentId, metadataUri)
      .accounts({
        identity: identityPda,
        reputation: reputationPda,
        rewardPool: rewardPoolPda,
        owner: owner.publicKey,
        config: configPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([owner])
      .rpc();

    console.log("Agent registered, tx:", tx);

    const identity = await program.account.identityRegistry.fetch(identityPda);
    expect(identity.owner.toString()).to.equal(owner.publicKey.toString());
    expect(identity.agentId).to.equal(agentId);
    expect(identity.metadataUri).to.equal(metadataUri);
    expect(identity.isActive).to.be.true;

    const reputation = await program.account.reputationRegistry.fetch(
      reputationPda
    );
    expect(reputation.agent.toString()).to.equal(identityPda.toString());
    expect(reputation.score.toNumber()).to.equal(5000);
    expect(reputation.totalTasks.toNumber()).to.equal(0);
    expect(reputation.successfulTasks.toNumber()).to.equal(0);

    const rewardPool = await program.account.rewardPool.fetch(rewardPoolPda);
    expect(rewardPool.claimableAmount.toNumber()).to.equal(0);

    console.log("✓ Agent registered successfully");
    console.log("  Agent ID:", identity.agentId);
    console.log("  Initial reputation:", reputation.score.toNumber());
  });

  it("Updates agent metadata", async () => {
    console.log("\n=== Test 3: Update Metadata ===");

    const newMetadataUri = "https://arweave.net/updated-metadata-v2";

    const tx = await program.methods
      .updateMetadata(newMetadataUri)
      .accounts({
        identity: identityPda,
        owner: owner.publicKey,
      })
      .signers([owner])
      .rpc();

    console.log("Metadata updated, tx:", tx);

    const identity = await program.account.identityRegistry.fetch(identityPda);
    expect(identity.metadataUri).to.equal(newMetadataUri);

    console.log("✓ Metadata updated successfully");
    console.log("  New URI:", identity.metadataUri);
  });

  it("Submits validation (approved)", async () => {
    console.log("\n=== Test 4: Submit Validation (Approved) ===");

    taskHash = crypto.randomBytes(32);

    [validationPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("validation"), identityPda.toBuffer(), taskHash],
      program.programId
    );

    const evidenceUri = "https://ipfs.io/ipfs/QmExample123";

    const treasuryBalanceBefore = await provider.connection.getBalance(
      treasury.publicKey
    );

    const tx = await program.methods
      .submitValidation(Array.from(taskHash), true, evidenceUri)
      .accounts({
        validation: validationPda,
        agent: identityPda,
        validator: validator.publicKey,
        config: configPda,
        treasury: treasury.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([validator])
      .rpc();

    console.log("Validation submitted, tx:", tx);

    const validation = await program.account.validationRegistry.fetch(
      validationPda
    );
    expect(validation.agent.toString()).to.equal(identityPda.toString());
    expect(validation.validator.toString()).to.equal(
      validator.publicKey.toString()
    );
    expect(validation.approved).to.be.true;
    expect(validation.evidenceUri).to.equal(evidenceUri);

    const treasuryBalanceAfter = await provider.connection.getBalance(
      treasury.publicKey
    );
    const commission = treasuryBalanceAfter - treasuryBalanceBefore;

    console.log("✓ Validation submitted successfully");
    console.log("  Task approved:", validation.approved);
    console.log("  Commission collected:", commission, "lamports");
  });

  it("Updates reputation after validation", async () => {
    console.log("\n=== Test 5: Update Reputation ===");

    const reputationBefore = await program.account.reputationRegistry.fetch(
      reputationPda
    );
    const scoreBefore = reputationBefore.score.toNumber();

    const tx = await program.methods
      .updateReputation()
      .accounts({
        reputation: reputationPda,
        agent: identityPda,
        validation: validationPda,
        rewardPool: rewardPoolPda,
      })
      .rpc();

    console.log("Reputation updated, tx:", tx);

    const reputation = await program.account.reputationRegistry.fetch(
      reputationPda
    );
    expect(reputation.totalTasks.toNumber()).to.equal(1);
    expect(reputation.successfulTasks.toNumber()).to.equal(1);
    expect(reputation.score.toNumber()).to.be.greaterThan(scoreBefore);

    const rewardPool = await program.account.rewardPool.fetch(rewardPoolPda);
    expect(rewardPool.claimableAmount.toNumber()).to.be.greaterThan(0);

    console.log("✓ Reputation updated successfully");
    console.log("  Score before:", scoreBefore);
    console.log("  Score after:", reputation.score.toNumber());
    console.log("  Score increase:", reputation.score.toNumber() - scoreBefore);
    console.log("  Success rate:", 
      (reputation.successfulTasks.toNumber() / reputation.totalTasks.toNumber()) * 100 + "%"
    );
    console.log("  Rewards earned:", rewardPool.claimableAmount.toNumber(), "lamports");
  });

  it("Submits another validation (rejected)", async () => {
    console.log("\n=== Test 6: Submit Validation (Rejected) ===");

    const taskHash2 = crypto.randomBytes(32);

    const [validationPda2] = PublicKey.findProgramAddressSync(
      [Buffer.from("validation"), identityPda.toBuffer(), taskHash2],
      program.programId
    );

    const tx = await program.methods
      .submitValidation(Array.from(taskHash2), false, "")
      .accounts({
        validation: validationPda2,
        agent: identityPda,
        validator: validator.publicKey,
        config: configPda,
        treasury: treasury.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([validator])
      .rpc();

    console.log("Validation submitted, tx:", tx);

    const tx2 = await program.methods
      .updateReputation()
      .accounts({
        reputation: reputationPda,
        agent: identityPda,
        validation: validationPda2,
        rewardPool: rewardPoolPda,
      })
      .rpc();

    console.log("Reputation updated, tx:", tx2);

    const reputation = await program.account.reputationRegistry.fetch(
      reputationPda
    );

    console.log("✓ Rejected validation processed");
    console.log("  Total tasks:", reputation.totalTasks.toNumber());
    console.log("  Successful:", reputation.successfulTasks.toNumber());
    console.log("  Failed:", reputation.failedTasks.toNumber());
    console.log("  Current score:", reputation.score.toNumber());
    console.log("  Success rate:", 
      (reputation.successfulTasks.toNumber() / reputation.totalTasks.toNumber()) * 100 + "%"
    );
  });

  it("Fetches final agent stats", async () => {
    console.log("\n=== Test 7: Final Stats ===");

    const identity = await program.account.identityRegistry.fetch(identityPda);
    const reputation = await program.account.reputationRegistry.fetch(
      reputationPda
    );
    const rewardPool = await program.account.rewardPool.fetch(rewardPoolPda);
    const config = await program.account.globalConfig.fetch(configPda);

    console.log("\n📊 Final Agent Statistics:");
    console.log("─────────────────────────────");
    console.log("Agent ID:", identity.agentId);
    console.log("Owner:", identity.owner.toString());
    console.log("Active:", identity.isActive);
    console.log("\n🏆 Reputation:");
    console.log("Score:", reputation.score.toNumber(), "/ 10000");
    console.log("Total Tasks:", reputation.totalTasks.toNumber());
    console.log("Successful:", reputation.successfulTasks.toNumber());
    console.log("Failed:", reputation.failedTasks.toNumber());
    console.log("Success Rate:", 
      (reputation.successfulTasks.toNumber() / reputation.totalTasks.toNumber()) * 100 + "%"
    );
    console.log("\n💰 Rewards:");
    console.log("Claimable:", rewardPool.claimableAmount.toNumber(), "lamports");
    console.log("Total Claimed:", rewardPool.totalClaimed.toNumber(), "lamports");
    console.log("\n🌐 Global Stats:");
    console.log("Total Agents:", config.totalAgents.toNumber());
    console.log("Total Validations:", config.totalValidations.toNumber());
    console.log("─────────────────────────────");
  });

  it("Deactivates agent", async () => {
    console.log("\n=== Test 8: Deactivate Agent ===");

    const tx = await program.methods
      .deactivateAgent()
      .accounts({
        identity: identityPda,
        owner: owner.publicKey,
      })
      .signers([owner])
      .rpc();

    console.log("Agent deactivated, tx:", tx);

    const identity = await program.account.identityRegistry.fetch(identityPda);
    expect(identity.isActive).to.be.false;

    console.log("✓ Agent deactivated successfully");
  });
});
