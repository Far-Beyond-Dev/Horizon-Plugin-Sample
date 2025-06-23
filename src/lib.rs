use async_trait::async_trait;
use event_system::{
    create_simple_plugin, current_timestamp, register_handlers, EventSystem, LogLevel,
    PlayerId, PluginError, Position, ServerContext, SimplePlugin,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, warn};

// ============================================================================
// Sample Plugin: Demonstrates core Horizon plugin functionality
// ============================================================================

/// Sample plugin demonstrating event handling, state management, and inter-plugin communication
pub struct SamplePlugin {
    name: String,
    // Plugin state - using Mutex for thread-safe access
    player_data: Arc<Mutex<HashMap<PlayerId, PlayerData>>>,
    config: PluginConfig,
}

/// Configuration for the plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub welcome_message: String,
    pub max_players_tracked: usize,
    pub enable_notifications: bool,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            welcome_message: "Welcome to the server!".to_string(),
            max_players_tracked: 100,
            enable_notifications: true,
        }
    }
}

/// Player data tracked by the plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    pub join_time: u64,
    pub last_position: Option<Position>,
    pub message_count: u32,
    pub jump_count: u32,
}

// ============================================================================
// Custom Events - Define your own events for inter-plugin communication
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerWelcomedEvent {
    pub player_id: PlayerId,
    pub welcome_message: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStatsEvent {
    pub player_id: PlayerId,
    pub messages_sent: u32,
    pub jumps_performed: u32,
    pub time_online: u64,
}

// ============================================================================
// Standard Events - Handle events from the server and other plugins
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerChatEvent {
    pub player_id: PlayerId,
    pub message: String,
    pub channel: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerJumpEvent {
    pub player_id: PlayerId,
    pub height: f64,
    pub position: Position,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMoveEvent {
    pub player_id: PlayerId,
    pub from_position: Position,
    pub to_position: Position,
    pub speed: f64,
}

impl SamplePlugin {
    pub fn new() -> Self {
        info!("🎯 SamplePlugin: Creating new instance");
        Self {
            name: "sample".to_string(),
            player_data: Arc::new(Mutex::new(HashMap::new())),
            config: PluginConfig::default(),
        }
    }

    /// Helper function to get or create player data
    fn get_or_create_player_data(&self, player_id: PlayerId) -> PlayerData {
        let mut data = self.player_data.lock().unwrap();
        data.entry(player_id)
            .or_insert_with(|| PlayerData {
                join_time: current_timestamp(),
                last_position: None,
                message_count: 0,
                jump_count: 0,
            })
            .clone()
    }

    /// Update player data
    fn update_player_data<F>(&self, player_id: PlayerId, updater: F)
    where
        F: FnOnce(&mut PlayerData),
    {
        let mut data = self.player_data.lock().unwrap();
        if let Some(player_data) = data.get_mut(&player_id) {
            updater(player_data);
        }
    }

    /// Get player statistics
    fn get_player_stats(&self, player_id: PlayerId) -> Option<PlayerStatsEvent> {
        let data = self.player_data.lock().unwrap();
        data.get(&player_id).map(|player_data| PlayerStatsEvent {
            player_id,
            messages_sent: player_data.message_count,
            jumps_performed: player_data.jump_count,
            time_online: current_timestamp() - player_data.join_time,
        })
    }
}

#[async_trait]
impl SimplePlugin for SamplePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    async fn register_handlers(&mut self, events: Arc<EventSystem>) -> Result<(), PluginError> {
        info!("🎯 SamplePlugin: Registering event handlers...");

        // Clone Arc references for use in closures
        let player_data = Arc::clone(&self.player_data);
        let config = self.config.clone();

        // ===== CORE SERVER EVENTS =====
        register_handlers!(events; core {
            // Handle player connections
            "player_connected" => move |event: serde_json::Value| {
                info!("🎯 SamplePlugin: Player connected! {:?}", event);
                
                if let Ok(player_id) = serde_json::from_value::<PlayerId>(event["player_id"].clone()) {
                    // Initialize player data
                    let mut data = player_data.lock().unwrap();
                    data.insert(player_id, PlayerData {
                        join_time: current_timestamp(),
                        last_position: None,
                        message_count: 0,
                        jump_count: 0,
                    });
                    
                    info!("🎯 SamplePlugin: Initialized data for player {}", player_id);
                }
                Ok(())
            },

            // Handle player disconnections
            "player_disconnected" => move |event: serde_json::Value| {
                info!("🎯 SamplePlugin: Player disconnected: {:?}", event);
                
                if let Ok(player_id) = serde_json::from_value::<PlayerId>(event["player_id"].clone()) {
                    // Clean up player data
                    let mut data = player_data.lock().unwrap();
                    if let Some(player_data) = data.remove(&player_id) {
                        let time_online = current_timestamp() - player_data.join_time;
                        info!("🎯 SamplePlugin: Player {} was online for {}s", player_id, time_online / 1000);
                    }
                }
                Ok(())
            }
        })?;

        // Clone again for client events
        let player_data = Arc::clone(&self.player_data);
        let config = self.config.clone();

        // ===== CLIENT EVENTS =====
        register_handlers!(events; client {
            // Handle chat messages
            "chat", "message" => move |event: PlayerChatEvent| {
                info!("🎯 SamplePlugin: Player {} said: '{}' in {}", 
                      event.player_id, event.message, event.channel);

                // Update message count
                let mut data = player_data.lock().unwrap();
                if let Some(player_data) = data.get_mut(&event.player_id) {
                    player_data.message_count += 1;
                }

                // Respond to specific commands
                if event.message.starts_with("!stats") {
                    info!("🎯 SamplePlugin: Player {} requested stats", event.player_id);
                    // Here you could emit an event to send a response back to the player
                }

                // Check for greeting
                if event.message.to_lowercase().contains("hello") ||
                   event.message.to_lowercase().contains("hi") {
                    if config.enable_notifications {
                        info!("🎯 SamplePlugin: Detected greeting from player {}", event.player_id);
                    }
                }

                Ok(())
            },

            // Handle player movement
            "movement", "position_update" => move |event: PlayerMoveEvent| {
                debug!("🎯 SamplePlugin: Player {} moved from {:?} to {:?}", 
                       event.player_id, event.from_position, event.to_position);

                // Update last known position
                let mut data = player_data.lock().unwrap();
                if let Some(player_data) = data.get_mut(&event.player_id) {
                    player_data.last_position = Some(event.to_position);
                }

                Ok(())
            },

            // Handle jump events
            "movement", "jump" => move |event: PlayerJumpEvent| {
                info!("🎯 SamplePlugin: Player {} jumped {:.1}m high! 🦘", 
                      event.player_id, event.height);

                // Update jump count
                let mut data = player_data.lock().unwrap();
                if let Some(player_data) = data.get_mut(&event.player_id) {
                    player_data.jump_count += 1;
                }

                // Special handling for high jumps
                if event.height > 5.0 {
                    info!("🎯 SamplePlugin: Impressive jump by player {}!", event.player_id);
                    // Could emit a special event for high jumps
                }

                Ok(())
            }
        })?;

        // ===== PLUGIN EVENTS =====
        register_handlers!(events; plugin {
            // Listen for events from other plugins
            "logger", "activity_logged" => |event: serde_json::Value| {
                debug!("🎯 SamplePlugin: Logger plugin recorded: {:?}", event);
                Ok(())
            },

            // Handle inventory events
            "inventory", "item_used" => |event: serde_json::Value| {
                info!("🎯 SamplePlugin: Player used item: {:?}", event);
                Ok(())
            }
        })?;

        info!("🎯 SamplePlugin: ✅ All handlers registered successfully!");
        Ok(())
    }

    async fn on_init(&mut self, context: Arc<dyn ServerContext>) -> Result<(), PluginError> {
        context.log(
            LogLevel::Info,
            "🎯 SamplePlugin: Starting up! Ready to demonstrate plugin functionality!",
        );

        // Load configuration (in a real plugin, you might load from a config file)
        info!("🎯 SamplePlugin: Loaded configuration: {:?}", self.config);

        // Announce our startup to other plugins
        let events = context.events();
        events
            .emit_plugin(
                "sample",
                "startup",
                &serde_json::json!({
                    "plugin": "sample",
                    "version": self.version(),
                    "message": "Sample plugin is now online and ready!",
                    "timestamp": current_timestamp(),
                    "features": [
                        "player_tracking",
                        "chat_monitoring", 
                        "movement_tracking",
                        "jump_counting"
                    ]
                }),
            )
            .await
            .map_err(|e| PluginError::InitializationFailed(e.to_string()))?;

        // Example: Request data from another plugin
        events
            .emit_plugin(
                "inventory",
                "get_system_info",
                &serde_json::json!({
                    "requester": "sample",
                    "timestamp": current_timestamp()
                }),
            )
            .await
            .map_err(|e| PluginError::InitializationFailed(e.to_string()))?;

        info!("🎯 SamplePlugin: ✅ Initialization complete!");
        Ok(())
    }

    async fn on_shutdown(&mut self, context: Arc<dyn ServerContext>) -> Result<(), PluginError> {
        let player_count = self.player_data.lock().unwrap().len();
        
        context.log(
            LogLevel::Info,
            &format!(
                "🎯 SamplePlugin: Shutting down. Tracked {} players during this session!",
                player_count
            ),
        );

        // Generate final statistics
        let data = self.player_data.lock().unwrap();
        let total_messages: u32 = data.values().map(|p| p.message_count).sum();
        let total_jumps: u32 = data.values().map(|p| p.jump_count).sum();

        info!("🎯 SamplePlugin: Session stats - Messages: {}, Jumps: {}", 
              total_messages, total_jumps);

        // Announce shutdown to other plugins
        let events = context.events();
        events
            .emit_plugin(
                "sample",
                "shutdown",
                &serde_json::json!({
                    "plugin": "sample",
                    "session_stats": {
                        "players_tracked": player_count,
                        "total_messages": total_messages,
                        "total_jumps": total_jumps
                    },
                    "message": "Sample plugin going offline. Thanks for the demonstration!",
                    "timestamp": current_timestamp()
                }),
            )
            .await
            .map_err(|e| PluginError::ExecutionError(e.to_string()))?;

        info!("🎯 SamplePlugin: ✅ Shutdown complete!");
        Ok(())
    }
}

// Create the plugin using the macro - zero unsafe code!
create_simple_plugin!(SamplePlugin);

// ============================================================================
// Utility Functions - Helper functions for common plugin tasks
// ============================================================================

/// Calculate distance between two positions
pub fn distance_between(pos1: &Position, pos2: &Position) -> f64 {
    let dx = pos1.x - pos2.x;
    let dy = pos1.y - pos2.y;
    let dz = pos1.z - pos2.z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Format time duration in a human-readable way
pub fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

// ============================================================================
// Tests - Example tests for your plugin
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_calculation() {
        let pos1 = Position { x: 0.0, y: 0.0, z: 0.0 };
        let pos2 = Position { x: 3.0, y: 4.0, z: 0.0 };
        assert_eq!(distance_between(&pos1, &pos2), 5.0);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(45), "45s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m 1s");
    }

    #[test]
    fn test_plugin_creation() {
        let plugin = SamplePlugin::new();
        assert_eq!(plugin.name(), "sample");
        assert_eq!(plugin.version(), "1.0.0");
    }
}