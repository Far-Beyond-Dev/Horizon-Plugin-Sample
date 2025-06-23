# Horizon Plugin Sample

🎯 **A comprehensive template for creating Horizon game server plugins**

This repository serves as the official template for Horizon plugin development. It demonstrates best practices, common patterns, and provides a solid foundation for building your own plugins.

> **Note**: This template is automatically used by the Far Beyond CLI (`fbcli`) when creating new plugins. You typically don't need to clone this directly - use `fbcli horizon plugin new <name>` instead.

## 🚀 Quick Start

### Using Far Beyond CLI (Recommended)

```bash
# Install fbcli first
cargo install fbcli

# Create a new plugin from this template
fbcli horizon plugin new my_awesome_plugin
cd my_awesome_plugin

# Build and deploy your plugin
fbcli horizon plugin build
```

### Manual Setup

```bash
git clone https://github.com/Far-Beyond-Dev/Horizon-Plugin-Sample.git my_plugin
cd my_plugin
rm -rf .git

# Update Cargo.toml with your plugin name
# Edit src/lib.rs with your plugin logic

cargo build --release
```

## 📚 What's Included

### Core Features Demonstrated

- **Event Handling**: Examples of handling core server events, client events, and plugin events
- **State Management**: Thread-safe player data tracking using `Arc<Mutex<>>`
- **Inter-Plugin Communication**: Sending and receiving events between plugins
- **Custom Events**: Defining your own event types
- **Configuration**: Plugin configuration patterns
- **Error Handling**: Proper error handling and logging
- **Testing**: Unit tests for plugin functionality

### Event Types Covered

- **Core Events**: `player_connected`, `player_disconnected`
- **Client Events**: `chat/message`, `movement/position_update`, `movement/jump`
- **Plugin Events**: Communication with other plugins like `inventory`, `logger`

### Best Practices Shown

- ✅ Proper use of the `create_simple_plugin!` macro
- ✅ Thread-safe state management
- ✅ Comprehensive logging with different levels
- ✅ Event registration patterns
- ✅ Clean shutdown procedures
- ✅ Plugin lifecycle management
- ✅ Error handling and recovery

## 🏗️ Plugin Structure

```rust
pub struct SamplePlugin {
    name: String,
    player_data: Arc<Mutex<HashMap<PlayerId, PlayerData>>>,
    config: PluginConfig,
}

impl SimplePlugin for SamplePlugin {
    // Required methods:
    fn name(&self) -> &str { /* ... */ }
    fn version(&self) -> &str { /* ... */ }
    async fn register_handlers(&mut self, events: Arc<EventSystem>) -> Result<(), PluginError> { /* ... */ }
    async fn on_init(&mut self, context: Arc<dyn ServerContext>) -> Result<(), PluginError> { /* ... */ }
    async fn on_shutdown(&mut self, context: Arc<dyn ServerContext>) -> Result<(), PluginError> { /* ... */ }
}
```

## 🎯 Key Concepts

### Event Registration

Use the `register_handlers!` macro to handle different types of events:

```rust
register_handlers!(events; core {
    "player_connected" => |event: serde_json::Value| {
        info!("Player joined: {:?}", event);
        Ok(())
    }
})?;

register_handlers!(events; client {
    "chat", "message" => |event: PlayerChatEvent| {
        info!("Chat: {}", event.message);
        Ok(())
    }
})?;

register_handlers!(events; plugin {
    "other_plugin", "some_event" => |event: CustomEvent| {
        info!("Received: {:?}", event);
        Ok(())
    }
})?;
```

### State Management

```rust
// Thread-safe state
let player_data: Arc<Mutex<HashMap<PlayerId, PlayerData>>> = 
    Arc::new(Mutex::new(HashMap::new()));

// Access in event handlers
let mut data = player_data.lock().unwrap();
data.insert(player_id, PlayerData::new());
```

### Inter-Plugin Communication

```rust
// Send events to other plugins
events.emit_plugin(
    "target_plugin",
    "event_name", 
    &serde_json::json!({
        "data": "value",
        "timestamp": current_timestamp()
    })
).await?;
```

## 🔧 Development Workflow

### 1. Customize Your Plugin

1. **Update Plugin Name**: Modify the struct name and `name()` method
2. **Define Your Events**: Create custom event types with `#[derive(Serialize, Deserialize)]`
3. **Implement Event Handlers**: Add your business logic in the `register_handlers` method
4. **Add State**: Define what data your plugin needs to track
5. **Configure Initialization**: Set up your plugin in `on_init`

### 2. Add Dependencies

Add any additional crates you need in `Cargo.toml`:

```toml
[dependencies]
# Core dependencies (already included)
event_system = { path = "../event_system" }
serde = { version = "1.0", features = ["derive"] }
# ... other core deps

# Your additional dependencies
regex = "1.0"
reqwest = "0.11"
sqlx = "0.7"
```

### 3. Testing

Run the included tests:

```bash
cargo test
```

Add your own tests following the examples in the `tests` module.

### 4. Building

```bash
# Development build
cargo build

# Release build (for production)
cargo build --release

# Using fbcli (handles everything automatically)
fbcli horizon plugin build
```

## 📖 Examples Directory

The `examples/` directory contains additional code samples:

- **`basic_events.rs`**: Minimal event handling examples
- **`custom_events.rs`**: Creating and using custom events
- **`inter_plugin.rs`**: Advanced inter-plugin communication patterns

## 🔍 Common Patterns

### Player Tracking

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    pub join_time: u64,
    pub last_position: Option<Position>,
    pub stats: PlayerStats,
}

// In your event handlers
fn handle_player_join(&self, player_id: PlayerId) {
    let mut data = self.player_data.lock().unwrap();
    data.insert(player_id, PlayerData::new());
}
```

### Configuration Management

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enable_feature: bool,
    pub max_value: i32,
    pub message_template: String,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enable_feature: true,
            max_value: 100,
            message_template: "Hello, {}!".to_string(),
        }
    }
}
```

### Error Handling

```rust
async fn risky_operation(&self) -> Result<(), PluginError> {
    some_operation()
        .map_err(|e| PluginError::ExecutionError(format!("Operation failed: {}", e)))?;
    Ok(())
}
```

## 🚨 Important Notes

### Thread Safety

- Always use `Arc<Mutex<>>` for shared state
- Be careful with lock ordering to avoid deadlocks
- Keep critical sections short

### Performance

- Use `debug!` for verbose logging that can be filtered out
- Avoid blocking operations in event handlers
- Consider using channels for heavy processing

### Event System

- Events are processed asynchronously
- Event handlers should return quickly
- Use `emit_plugin` for inter-plugin communication
- Custom events must implement `Serialize + Deserialize`

## 📝 License

This template is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Found an issue or want to improve this template? 

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

---

**Happy plugin development!** 🎉