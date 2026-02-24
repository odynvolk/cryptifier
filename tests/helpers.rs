use std::fs;
use std::path::Path;

const FIXTURES_DIR: &str = "tests/data";

pub fn load_fixture(name: &str) -> serde_json::Value {
    let path = Path::new(FIXTURES_DIR).join(format!("{}.json", name));
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to load fixture {:?}: {}", path, e));
    serde_json::from_str(&content).unwrap_or_else(|e| panic!("Failed to parse fixture {:?}: {}", path, e))
}

pub fn get_fng_fixture() -> serde_json::Value {
    load_fixture("alternative-me")
}

pub fn get_bitnodes_fixture() -> serde_json::Value {
    load_fixture("bitnodes")
}

pub fn get_cbbi_fixture() -> serde_json::Value {
    load_fixture("cbbi")
}

pub fn get_coin_gecko_fixture(id: &str) -> Option<std::collections::HashMap<String, cryptifier::sources::coin_gecko::CoinPrice>> {
    let data = load_fixture("coin_gecko");
    if let Some(obj) = data.as_object() {
        if let Some(coin_data) = obj.get(id) {
            if let Some(coin_obj) = coin_data.as_object() {
                let mut result = std::collections::HashMap::new();
                if let Some(serde_json::Value::Number(usd)) = coin_obj.get("usd") {
                    let price = cryptifier::sources::coin_gecko::CoinPrice {
                        usd: usd.as_f64(),
                    };
                    result.insert(id.to_string(), price);
                }
                return Some(result);
            }
        }
    }
    None
}
