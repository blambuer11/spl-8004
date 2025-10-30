# SPL-8004: Trustless AI Agent Identity & Reputation Standard

Solana'da AI ajanları için merkezi olmayan kimlik, itibar ve doğrulama standardı. ERC-8004'ün Solana implementasyonu.

## 🚀 Özellikler

- **Identity Registry**: On-chain ajan kimlik yönetimi
- **Reputation System**: Görev tabanlı itibar skorlama (0-10000)
- **Validation Registry**: Trustless görev doğrulama mekanizması
- **Reward System**: İtibar bazlı ödül havuzu
- **Commission System**: Doğrulama komisyon mekanizması (%1-10)
- **SDK Support**: TypeScript ve Rust client'ları

## 📦 Kurulum

### Gereksinimler

```bash
# Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.18.0/install)"

# Anchor Framework
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.30.1
avm use 0.30.1

# Node.js dependencies
yarn install
```

### Build

```bash
# Program build
anchor build

# Program ID'yi güncelle
anchor keys list
# Çıktıdaki program ID'yi lib.rs ve Anchor.toml'de güncelle

# Tekrar build
anchor build
```

## 🧪 Test

### Localnet

```bash
# Localnet başlat
solana-test-validator

# Test çalıştır
anchor test --skip-local-validator
```

### Devnet

```bash
# Devnet'e geç
solana config set --url devnet

# Airdrop al
solana airdrop 2

# Deploy
anchor deploy --provider.cluster devnet

# Test
anchor test --provider.cluster devnet
```

## 🔧 Kullanım

### JavaScript/TypeScript SDK

```typescript
import { SPL8004Client } from "./client/sdk";
import { Connection, Keypair } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";

// Client oluştur
const connection = new Connection("https://api.devnet.solana.com");
const wallet = new anchor.Wallet(Keypair.generate());
const client = new SPL8004Client(connection, wallet);

// Config initialize (sadece bir kez)
await client.initializeConfig(
  300, // 3% commission
  treasury.publicKey
);

// Ajan kaydet
const { identityPda, reputationPda } = await client.registerAgent(
  "agent-123",
  "https://arweave.net/metadata"
);

// Doğrulama gönder
const taskHash = Buffer.from(crypto.randomBytes(32));
const { validationPda } = await client.submitValidation(
  "agent-123",
  taskHash,
  true, // approved
  "https://ipfs.io/evidence"
);

// İtibari güncelle
await client.updateReputation("agent-123", taskHash);

// İtibar skorunu getir
const reputation = await client.getReputation("agent-123");
console.log("Score:", reputation.score.toNumber());
console.log("Success Rate:", 
  (reputation.successfulTasks.toNumber() / reputation.totalTasks.toNumber()) * 100 + "%"
);

// Ödülleri claim et
await client.claimRewards("agent-123");
```

### Direct Anchor Usage

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";

const program = anchor.workspace.Spl8004;

// PDA'ları bul
const [identityPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("identity"), Buffer.from("agent-123")],
  program.programId
);

// Ajan kaydet
await program.methods
  .registerAgent("agent-123", "https://arweave.net/metadata")
  .accounts({
    identity: identityPda,
    // ... diğer accounts
  })
  .rpc();
```

## 📊 Account Yapıları

### IdentityRegistry

```rust
pub struct IdentityRegistry {
    pub owner: Pubkey,          // Ajan sahibi
    pub agent_id: String,       // Benzersiz ID (max 64)
    pub metadata_uri: String,   // Metadata URI (max 200)
    pub created_at: i64,        // Oluşturulma zamanı
    pub updated_at: i64,        // Güncelleme zamanı
    pub is_active: bool,        // Aktif durumu
    pub bump: u8,               // PDA bump
}
```

### ReputationRegistry

```rust
pub struct ReputationRegistry {
    pub agent: Pubkey,              // Ajan identity
    pub score: u64,                 // İtibar skoru (0-10000)
    pub total_tasks: u64,           // Toplam görev
    pub successful_tasks: u64,      // Başarılı görev
    pub failed_tasks: u64,          // Başarısız görev
    pub last_updated: i64,          // Son güncelleme
    pub stake_amount: u64,          // Stake miktarı
    pub bump: u8,
}
```

### ValidationRegistry

```rust
pub struct ValidationRegistry {
    pub agent: Pubkey,          // Doğrulanan ajan
    pub validator: Pubkey,      // Doğrulayıcı
    pub task_hash: [u8; 32],    // Görev hash'i
    pub approved: bool,         // Onay durumu
    pub timestamp: i64,         // Zaman damgası
    pub evidence_uri: String,   // Kanıt URI'si
    pub bump: u8,
}
```

## 🎯 Instructions

| Instruction | Açıklama | Parametreler |
|-------------|----------|--------------|
| `initialize_config` | Global config oluştur | commission_rate, treasury |
| `register_agent` | Yeni ajan kaydet | agent_id, metadata_uri |
| `update_metadata` | Metadata güncelle | new_metadata_uri |
| `submit_validation` | Doğrulama gönder | task_hash, approved, evidence_uri |
| `update_reputation` | İtibarı güncelle | - |
| `deactivate_agent` | Ajanı deaktive et | - |
| `claim_rewards` | Ödülleri claim et | - |

## 💰 İtibar Sistemi

### Skor Hesaplama

**Başarılı görev (approved=true):**
- Success rate 90-100%: +100 puan
- Success rate 80-89%: +75 puan
- Success rate 70-79%: +50 puan
- Success rate <70%: +25 puan

**Başarısız görev (approved=false):**
- Success rate 0-50%: -150 puan
- Success rate 51-70%: -100 puan
- Success rate >70%: -50 puan

### Ödül Sistemi

```rust
// Base reward: 0.0001 SOL
// Multiplier based on score:
9000-10000: 5x
8000-8999:  4x
7000-7999:  3x
6000-6999:  2x
<6000:      1x
```

## 🔐 Güvenlik

### PDA Seeds

```rust
CONFIG:      ["config"]
IDENTITY:    ["identity", agent_id]
REPUTATION:  ["reputation", identity_pda]
VALIDATION:  ["validation", identity_pda, task_hash]
REWARD_POOL: ["reward_pool", identity_pda]
```

### Access Control

- Sadece owner metadata güncelleyebilir
- Sadece owner ajanı deaktive edebilir
- Sadece owner ödül claim edebilir
- Herhangi bir validator doğrulama gönderebilir

## 📈 Performans

- **TPS**: 65,000+ transaction/saniye (Solana)
- **Maliyet**: ~$0.00025 per transaction
- **Latency**: 400ms confirmation time
- **Skalabilite**: Sınırsız ajan desteği (PDA'lar)

## 🌐 Deploy

### Devnet

```bash
solana config set --url devnet
anchor build
anchor deploy --provider.cluster devnet

# Program ID
solana program show <PROGRAM_ID> --url devnet
```

### Mainnet

```bash
solana config set --url mainnet-beta
anchor build --verifiable

# Deploy
anchor deploy --provider.cluster mainnet-beta

# Verify
solana program show <PROGRAM_ID> --url mainnet-beta

# İsteğe bağlı: Upgrade authority'yi kaldır (final release)
solana program set-upgrade-authority <PROGRAM_ID> --final
```

## 🤝 Katkıda Bulunma

1. Fork yapın
2. Feature branch oluşturun (`git checkout -b feature/amazing`)
3. Commit yapın (`git commit -m 'Add amazing feature'`)
4. Push yapın (`git push origin feature/amazing`)
5. Pull Request açın

## 📝 Lisans

MIT License - detaylar için [LICENSE](LICENSE) dosyasına bakın.

## 🔗 Bağlantılar

- **Solana Docs**: https://docs.solana.com
- **Anchor Docs**: https://www.anchor-lang.com
- **ERC-8004 Spec**: https://eips.ethereum.org/EIPS/eip-8004
- **Discord**: [Community link]
- **Twitter**: [@SPL8004]

## 🎓 Kaynaklar

- [Solana Program Examples](https://github.com/solana-labs/solana-program-library)
- [Anchor Book](https://book.anchor-lang.com)
- [Solana Cookbook](https://solanacookbook.com)

## ⚠️ Disclaimer

Bu yazılım "olduğu gibi" sağlanmaktadır. Production kullanımından önce security audit yapılması önerilir.

---

**Built with ❤️ for Solana AI Agents**
