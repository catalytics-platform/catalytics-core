use std::collections::HashMap;

#[derive(Debug)]
pub struct WalletHoldings {
    pub public_key: String,
    pub token_holdings: HashMap<String, f64>
}
