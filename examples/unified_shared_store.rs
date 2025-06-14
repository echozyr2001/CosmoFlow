use std::collections::HashMap;

use cosmoflow::shared_store::new_design::SharedStore;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// A simple in-memory storage implementation
#[derive(Debug, Clone)]
pub struct SimpleStorage {
    data: HashMap<String, serde_json::Value>,
}

impl SimpleStorage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Default for SimpleStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedStore for SimpleStorage {
    type Error = SimpleStorageError;

    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error> {
        match self.data.get(key) {
            Some(value) => {
                let deserialized = serde_json::from_value(value.clone())
                    .map_err(|e| SimpleStorageError::DeserializationError(e.to_string()))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    fn set<T: Serialize>(&mut self, key: String, value: T) -> Result<(), Self::Error> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| SimpleStorageError::SerializationError(e.to_string()))?;
        self.data.insert(key, json_value);
        Ok(())
    }

    fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, Self::Error> {
        match self.data.remove(key) {
            Some(value) => {
                let deserialized = serde_json::from_value(value)
                    .map_err(|e| SimpleStorageError::DeserializationError(e.to_string()))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    fn contains_key(&self, key: &str) -> Result<bool, Self::Error> {
        Ok(self.data.contains_key(key))
    }

    fn keys(&self) -> Result<Vec<String>, Self::Error> {
        Ok(self.data.keys().cloned().collect())
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.data.clear();
        Ok(())
    }

    fn len(&self) -> Result<usize, Self::Error> {
        Ok(self.data.len())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SimpleStorageError {
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct UserData {
    id: u64,
    name: String,
    active: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CosmoFlow: Unified SharedStore Trait Demo ===\n");

    // Create a memory storage that directly implements SharedStore trait
    let mut store = SimpleStorage::new();

    // Store different types of data
    println!("1. Storing data using the unified SharedStore trait:");

    let user = UserData {
        id: 42,
        name: "Alice".to_string(),
        active: true,
    };

    store.set("user".to_string(), &user)?;
    store.set("counter".to_string(), 100u32)?;
    store.set("message".to_string(), "Hello, unified design!".to_string())?;

    println!("   ✓ Stored user data: {user:?}");
    println!("   ✓ Stored counter: 100");
    println!("   ✓ Stored message: 'Hello, unified design!'");

    // Retrieve data
    println!("\n2. Retrieving data:");

    let retrieved_user: Option<UserData> = store.get("user")?;
    let retrieved_counter: Option<u32> = store.get("counter")?;
    let retrieved_message: Option<String> = store.get("message")?;

    println!("   ✓ Retrieved user: {retrieved_user:?}");
    println!("   ✓ Retrieved counter: {retrieved_counter:?}");
    println!("   ✓ Retrieved message: {retrieved_message:?}");

    // Demonstrate unified interface methods
    println!("\n3. Using unified interface methods:");

    println!("   ✓ Number of items: {}", store.len()?);
    println!("   ✓ Contains 'user' key: {}", store.contains_key("user")?);
    println!("   ✓ All keys: {:?}", store.keys()?);
    println!("   ✓ Is empty: {}", store.is_empty()?);

    // Remove an item
    println!("\n4. Removing data:");
    let removed_counter: Option<u32> = store.remove("counter")?;
    println!("   ✓ Removed counter: {removed_counter:?}");
    println!("   ✓ Remaining items: {}", store.len()?);

    // Clear all data
    println!("\n5. Clearing all data:");
    store.clear()?;
    println!("   ✓ Cleared all data");
    println!("   ✓ Is empty: {}", store.is_empty()?);

    println!("\n=== Demo completed successfully! ===");
    println!("\nKey benefits of the unified design:");
    println!("• Single trait interface reduces complexity");
    println!("• Direct implementation eliminates wrapper overhead");
    println!("• Maintains all original functionality");
    println!("• Simplifies API surface for users");

    Ok(())
}
