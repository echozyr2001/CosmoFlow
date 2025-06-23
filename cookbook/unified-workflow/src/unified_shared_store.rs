use cosmoflow::shared_store::{backends::MemoryStorage, SharedStore};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct UserData {
    id: u64,
    name: String,
    active: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CosmoFlow: Unified SharedStore Trait Demo ===\n");

    // Create shared storage with MemoryStorage backend
    let mut store = MemoryStorage::new();

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
