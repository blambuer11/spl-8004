import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { Connection, PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { Spl8004 } from "../target/types/spl_8004";

export class SPL8004Client {
  private program: Program<Spl8004>;
  private provider: AnchorProvider;

  constructor(
    connection: Connection,
    wallet: anchor.Wallet,
    programId?: PublicKey
  ) {
    this.provider = new AnchorProvider(connection, wallet, {
      commitment: "confirmed",
    });
    
    const idl = require("../target/idl/spl_8004.json");
    const pid = programId || new PublicKey("SPL8wVx7ZqKNxJk5H2bF8QyGvM4tN3rP9WdE6fU5Kc2");
    
    this.program = new Program(idl, pid, this.provider);
  }

  findConfigPda(): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      this.program.programId
    );
  }

  findIdentityPda(agentId: string): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("identity"), Buffer.from(agentId)],
      this.program.programId
    );
  }

  findReputationPda(identityPda: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("reputation"), identityPda.toBuffer()],
      this.program.programId
    );
  }

  findRewardPoolPda(identityPda: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("reward_pool"), identityPda.toBuffer()],
      this.program.programId
    );
  }

  findValidationPda(
    identityPda: PublicKey,
    taskHash: Buffer
  ): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("validation"), identityPda.toBuffer(), taskHash],
      this.program.programId
    );
  }

  async initializeConfig(
    commissionRate: number,
    treasury: PublicKey
  ): Promise<string> {
    const [configPda] = this.findConfigPda();

    const tx = await this.program.methods
      .initializeConfig(commissionRate, treasury)
      .accounts({
        config: configPda,
        authority: this.provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    return tx;
  }

  async registerAgent(
    agentId: string,
    metadataUri: string,
    owner?: Keypair
  ): Promise<{
    signature: string;
    identityPda: PublicKey;
    reputationPda: PublicKey;
    rewardPoolPda: PublicKey;
  }> {
    const [identityPda] = this.findIdentityPda(agentId);
    const [reputationPda] = this.findReputationPda(identityPda);
    const [rewardPoolPda] = this.findRewardPoolPda(identityPda);
    const [configPda] = this.findConfigPda();

    const ownerPubkey = owner
      ? owner.publicKey
      : this.provider.wallet.publicKey;

    const txBuilder = this.program.methods
      .registerAgent(agentId, metadataUri)
      .accounts({
        identity: identityPda,
        reputation: reputationPda,
        rewardPool: rewardPoolPda,
        owner: ownerPubkey,
        config: configPda,
        systemProgram: SystemProgram.programId,
      });

    const signature = owner
      ? await txBuilder.signers([owner]).rpc()
      : await txBuilder.rpc();

    return {
      signature,
      identityPda,
      reputationPda,
      rewardPoolPda,
    };
  }

  async updateMetadata(
    agentId: string,
    newMetadataUri: string,
    owner?: Keypair
  ): Promise<string> {
    const [identityPda] = this.findIdentityPda(agentId);
    const ownerPubkey = owner
      ? owner.publicKey
      : this.provider.wallet.publicKey;

    const txBuilder = this.program.methods
      .updateMetadata(newMetadataUri)
      .accounts({
        identity: identityPda,
        owner: ownerPubkey,
      });

    return owner ? await txBuilder.signers([owner]).rpc() : await txBuilder.rpc();
  }

  async submitValidation(
    agentId: string,
    taskHash: Buffer,
    approved: boolean,
    evidenceUri: string,
    validator?: Keypair
  ): Promise<{
    signature: string;
    validationPda: PublicKey;
  }> {
    const [identityPda] = this.findIdentityPda(agentId);
    const [validationPda] = this.findValidationPda(identityPda, taskHash);
    const [configPda] = this.findConfigPda();

    const config = await this.program.account.globalConfig.fetch(configPda);
    const validatorPubkey = validator
      ? validator.publicKey
      : this.provider.wallet.publicKey;

    const txBuilder = this.program.methods
      .submitValidation(Array.from(taskHash), approved, evidenceUri)
      .accounts({
        validation: validationPda,
        agent: identityPda,
        validator: validatorPubkey,
        config: configPda,
        treasury: config.treasury,
        systemProgram: SystemProgram.programId,
      });

    const signature = validator
      ? await txBuilder.signers([validator]).rpc()
      : await txBuilder.rpc();

    return {
      signature,
      validationPda,
    };
  }

  async updateReputation(
    agentId: string,
    taskHash: Buffer
  ): Promise<string> {
    const [identityPda] = this.findIdentityPda(agentId);
    const [reputationPda] = this.findReputationPda(identityPda);
    const [validationPda] = this.findValidationPda(identityPda, taskHash);
    const [rewardPoolPda] = this.findRewardPoolPda(identityPda);

    const tx = await this.program.methods
      .updateReputation()
      .accounts({
        reputation: reputationPda,
        agent: identityPda,
        validation: validationPda,
        rewardPool: rewardPoolPda,
      })
      .rpc();

    return tx;
  }

  async deactivateAgent(agentId: string, owner?: Keypair): Promise<string> {
    const [identityPda] = this.findIdentityPda(agentId);
    const ownerPubkey = owner
      ? owner.publicKey
      : this.provider.wallet.publicKey;

    const txBuilder = this.program.methods
      .deactivateAgent()
      .accounts({
        identity: identityPda,
        owner: ownerPubkey,
      });

    return owner ? await txBuilder.signers([owner]).rpc() : await txBuilder.rpc();
  }

  async claimRewards(agentId: string, owner?: Keypair): Promise<string> {
    const [identityPda] = this.findIdentityPda(agentId);
    const [rewardPoolPda] = this.findRewardPoolPda(identityPda);
    const ownerPubkey = owner
      ? owner.publicKey
      : this.provider.wallet.publicKey;

    const txBuilder = this.program.methods
      .claimRewards()
      .accounts({
        identity: identityPda,
        rewardPool: rewardPoolPda,
        owner: ownerPubkey,
        systemProgram: SystemProgram.programId,
      });

    return owner ? await txBuilder.signers([owner]).rpc() : await txBuilder.rpc();
  }

  async getIdentity(agentId: string) {
    const [identityPda] = this.findIdentityPda(agentId);
    return this.program.account.identityRegistry.fetch(identityPda);
  }

  async getReputation(agentId: string) {
    const [identityPda] = this.findIdentityPda(agentId);
    const [reputationPda] = this.findReputationPda(identityPda);
    return this.program.account.reputationRegistry.fetch(reputationPda);
  }

  async getRewardPool(agentId: string) {
    const [identityPda] = this.findIdentityPda(agentId);
    const [rewardPoolPda] = this.findRewardPoolPda(identityPda);
    return this.program.account.rewardPool.fetch(rewardPoolPda);
  }

  async getValidation(agentId: string, taskHash: Buffer) {
    const [identityPda] = this.findIdentityPda(agentId);
    const [validationPda] = this.findValidationPda(identityPda, taskHash);
    return this.program.account.validationRegistry.fetch(validationPda);
  }

  async getConfig() {
    const [configPda] = this.findConfigPda();
    return this.program.account.globalConfig.fetch(configPda);
  }

  async getAllAgents() {
    return this.program.account.identityRegistry.all();
  }

  async getAgentsByOwner(owner: PublicKey) {
    return this.program.account.identityRegistry.all([
      {
        memcmp: {
          offset: 8,
          bytes: owner.toBase58(),
        },
      },
    ]);
  }
}

export * from "../target/types/spl_8004";
