use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use ed25519_dalek::{SigningKey, Verifier, Signature, VerifyingKey, Signer};
use rand::rngs::OsRng;
use rand::RngCore;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use axum::{routing::{get, post}, Json, Router, extract::State};
use tower_http::cors::{CorsLayer, Any};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,    
    pub receiver: String,  
    pub amount: u32,
    pub nonce: u64,       
    pub signature: Vec<u8>,
}

impl Transaction {
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let input = format!("{}{}{}{}", self.sender, self.receiver, self.amount, self.nonce);
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    pub fn sign(&mut self, signing_key: &SigningKey) {
        let message = self.calculate_hash();
        let signature = signing_key.sign(message.as_bytes());
        self.signature = signature.to_bytes().to_vec();
    }
    pub fn is_signature_valid(&self, public_key_bytes: &[u8]) -> bool {
        if self.signature.is_empty() { return false; }
        let Ok(public_key) = VerifyingKey::from_bytes(public_key_bytes.try_into().unwrap_or(&[0u8;32])) else { return false; };
        let Ok(signature) = Signature::from_slice(self.signature.as_slice()) else { return false; };
        let message = self.calculate_hash();
        public_key.verify(message.as_bytes(), &signature).is_ok()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,             
    pub timestamp: u128,        
    pub transactions: Vec<Transaction>, 
    pub previous_hash: String,  
    pub hash: String,           
    pub nonce: u64,             
}

impl Block {
    pub fn genesis() -> Self {
        let mut block = Block {
            index: 0, timestamp: 1715000000, transactions: Vec::new(),
            previous_hash: String::from("0"), hash: String::new(), nonce: 0,
        };
        block.hash = block.calculate_hash();
        block
    }
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let input = format!("{}{}{:?}{}{}", self.index, self.timestamp, self.transactions, self.previous_hash, self.nonce);
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    pub fn mine_block(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);
        self.hash = self.calculate_hash();
        while self.hash.len() < difficulty || &self.hash[..difficulty] != target {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
    }
}

pub struct Blockchain {
    pub chain: Vec<Block>,          
    pub pending_transactions: Vec<Transaction>, 
    pub difficulty: usize,          
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Self {
        Blockchain {
            chain: vec![Block::genesis()],
            pending_transactions: Vec::new(),
            difficulty,
        }
    }
    pub fn add_transaction(&mut self, tx: Transaction, public_key: &[u8]) -> bool {
        let new_tx_hash = tx.calculate_hash();
        for block in &self.chain {
            if block.transactions.iter().any(|t| t.calculate_hash() == new_tx_hash) { return false; }
        }
        if self.pending_transactions.iter().any(|t| t.calculate_hash() == new_tx_hash) { return false; }
        if !tx.is_signature_valid(public_key) { return false; }
        self.pending_transactions.push(tx);
        true
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shared_state = Arc::new(Mutex::new(Blockchain::new(2)));
    
    let args: Vec<String> = std::env::args().collect();
    let port = if args.len() > 2 { &args[2] } else { "8080" };
    let web_port: u16 = port.parse::<u16>().unwrap_or(8080) + 1000;

    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    let app = Router::new()
        .route("/api/auctions", get(api_list_auctions).post(api_create_auction))
        .route("/api/mine", post(api_mine))
        .route("/api/bids", post(api_bid))
        .route("/api/test_block", get(api_test_block)) 
        .route("/api/close_auction", post(api_close_auction))
        .layer(cors)
        .with_state(shared_state.clone());

    println!("🚀 API Web a rodar na porta: {}", web_port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", web_port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Deserialize)]
struct CreateAuctionInput { item: String, min_bid: u32 }
#[derive(Deserialize)]
struct BidInput { auction_id: String, amount: u32, sender_type: String }
#[derive(Deserialize)]
struct CloseAuctionInput { auction_id: String, sender_type: String }

async fn api_list_auctions(State(state): State<Arc<Mutex<Blockchain>>>) -> Json<Value> {
    let coin = state.lock().unwrap();
    let mut todas_txs = Vec::new();
    for block in &coin.chain { todas_txs.extend(block.transactions.clone()); }
    todas_txs.extend(coin.pending_transactions.clone());

    let mut fechados = Vec::new();
    for tx in &todas_txs {
        let receiver_lower = tx.receiver.to_lowercase().trim().to_string();
        if receiver_lower.starts_with("close_auction:") {
            let id = receiver_lower.replace("close_auction:", "").trim().to_string();
            fechados.push(id);
        }
    }

    let mut auctions = Vec::new();
    for tx in &todas_txs {
        let receiver_lower = tx.receiver.to_lowercase().trim().to_string();
        if receiver_lower.starts_with("auction_start:") {
            let auc_id = receiver_lower.replace("auction_start:", "").trim().to_string();
            
            if !fechados.contains(&auc_id) {
                let mut current_highest = tx.amount;
                for bid_tx in &todas_txs {
                    if bid_tx.receiver.to_lowercase().trim() == auc_id && bid_tx.amount > current_highest {
                        current_highest = bid_tx.amount;
                    }
                }
                auctions.push(serde_json::json!({
                    "id": auc_id, "item": tx.sender.clone(), "min_bid": tx.amount, "current_bid": current_highest
                }));
            }
        }
    }
    Json(serde_json::json!(auctions))
}

async fn api_create_auction(State(state): State<Arc<Mutex<Blockchain>>>, Json(payload): Json<CreateAuctionInput>) -> Json<Value> {
    let mut secret_bytes = [0u8; 32]; OsRng.fill_bytes(&mut secret_bytes);
    let sk = SigningKey::from_bytes(&secret_bytes);
    let item = payload.item.clone();
    let auction_id = format!("{:x}", Sha256::digest(item.as_bytes()))[..8].to_lowercase().trim().to_string();
    
    let mut tx = Transaction {
        sender: item,
        receiver: format!("auction_start:{}", auction_id),
        amount: payload.min_bid,
        nonce: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64,
        signature: vec![],
    }; 
    
    tx.sign(&sk);
    
    let mut coin = state.lock().unwrap();
    coin.add_transaction(tx, &sk.verifying_key().to_bytes());
    Json(serde_json::json!({ "status": "success", "auction_id": auction_id }))
}

async fn api_bid(State(state): State<Arc<Mutex<Blockchain>>>, Json(payload): Json<BidInput>) -> Json<Value> {
    if !payload.sender_type.contains("Comprador") {
        return Json(serde_json::json!({ "status": "error", "message": "Apenas compradores podem licitar!" }));
    }

    let mut secret_bytes = [0u8; 32]; OsRng.fill_bytes(&mut secret_bytes);
    let sk = SigningKey::from_bytes(&secret_bytes);
    
    let mut tx = Transaction {
        sender: payload.sender_type, 
        receiver: payload.auction_id.to_lowercase().trim().to_string(), 
        amount: payload.amount,
        nonce: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64, 
        signature: vec![],
    };
    tx.sign(&sk);

    let mut coin = state.lock().unwrap();
    let sucesso = coin.add_transaction(tx, &sk.verifying_key().to_bytes());
    
    Json(serde_json::json!({ "status": if sucesso { "success" } else { "error" } }))
}

async fn api_mine(State(state): State<Arc<Mutex<Blockchain>>>) -> Json<Value> {
    let mut coin = state.lock().unwrap();
    let last_block = coin.chain.last().unwrap().clone();
    
    // Capturar o índice do novo bloco
    let new_index = coin.chain.len() as u64;
    
    println!("\n==================================================");
    println!("⛏️  [MINERAÇÃO] Iniciada a mineração do Bloco #{}", new_index);
    println!("📦 Transações a incluir neste bloco: {}", coin.pending_transactions.len());
    
    // Listar as transações pendentes que vão entrar para o terminal saber o que se passa
    for (i, tx) in coin.pending_transactions.iter().enumerate() {
        println!("  -> Tx [{}]: Remetente: '{}' | Destinatário: '{}' | Valor: {} USD", 
            i, tx.sender, tx.receiver, tx.amount
        );
    }

    let mut new_block = Block {
        index: new_index,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
        transactions: coin.pending_transactions.clone(),
        previous_hash: last_block.hash.clone(),
        hash: String::new(), 
        nonce: 0,
    };
    
    println!("⛓️  Hash do bloco anterior: {}", last_block.hash);
    println!("⏳ A computar Proof of Work (Dificuldade: {})...", coin.difficulty);
    
    // Executa o algoritmo de mineração (Loop do Nonce)
    new_block.mine_block(coin.difficulty);
    
    println!("✅ [BLOCO MINERADO COM SUCESSO!]");
    println!("   🔹 Índice: #{}", new_block.index);
    println!("   🔹 Nonce encontrado: {}", new_block.nonce);
    println!("   🔹 Hash do Novo Bloco: {}", new_block.hash);
    println!("==================================================\n");

    // Adiciona à cadeia e limpa a pool de pendentes
    coin.chain.push(new_block.clone());
    coin.pending_transactions.clear();
    
    Json(serde_json::json!({"status": "success", "block_index": new_block.index}))
}

async fn api_test_block(State(state): State<Arc<Mutex<Blockchain>>>) -> Json<Value> {
    let coin = state.lock().unwrap();
    let last_block = coin.chain.last().unwrap();
    
    // Adiciona este log para veres no terminal do Docker
    println!("\n🔍 [AUDITORIA] Teste de Integridade Solicitado...");
    println!("📦 Tamanho da Cadeia: {} blocos", coin.chain.len());
    println!("🔗 Hash do Último Bloco: {}", last_block.hash);
    
    let is_valid = !last_block.hash.is_empty();
    
    Json(serde_json::json!({
        "status": "success",
        "healthy": is_valid,
        "chain_length": coin.chain.len(),
        "last_block_hash": last_block.hash
    }))
}

async fn api_close_auction(State(state): State<Arc<Mutex<Blockchain>>>, Json(payload): Json<CloseAuctionInput>) -> Json<Value> {
    if !payload.sender_type.contains("Vendedor") {
        return Json(serde_json::json!({ "status": "error", "message": "Apenas o Vendedor pode fechar leilões!" }));
    }

    let tx = Transaction {
        sender: payload.sender_type,
        receiver: format!("close_auction:{}", payload.auction_id.to_lowercase().trim()),
        amount: 0,
        nonce: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64,
        signature: vec![],
    };

    let mut coin = state.lock().unwrap();
    coin.pending_transactions.push(tx); 
    
    // Log para auditoria no terminal do Docker
    println!("✅ [SUCESSO] Leilão {} marcado para encerramento pelo utilizador.", payload.auction_id);
    
    Json(serde_json::json!({ "status": "success", "message": "Leilão enviado para fecho." }))
}