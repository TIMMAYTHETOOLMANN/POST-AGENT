// State Tracker Module
// Tracks the state of posted content

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{read_to_string, write};
use std::path::Path;

/// State of posted content
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostState {
    /// Map of video IDs to posting status
    posted_ids: HashMap<String, bool>,
}

impl PostState {
    /// Load the state from disk
    pub fn load() -> Self {
        debug!("Loading post state from disk");

        match read_to_string("state.json") {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(state) => {
                    debug!("Loaded post state with {} entries", state.posted_ids.len());
                    state
                }
                Err(e) => {
                    error!("Failed to parse state file: {}", e);
                    Self::default()
                }
            },
            Err(e) => {
                warn!("Failed to read state file, creating new state: {}", e);
                Self::default()
            }
        }
    }

    /// Save the state to disk
    pub fn save(&self) {
        debug!("Saving post state to disk");

        match serde_json::to_string_pretty(self) {
            Ok(content) => match write("state.json", content) {
                Ok(_) => debug!("Successfully saved post state"),
                Err(e) => error!("Failed to write state file: {}", e),
            },
            Err(e) => error!("Failed to serialize state: {}", e),
        }
    }

    /// Mark a video as posted
    pub fn mark_posted(&mut self, id: &str) {
        debug!("Marking video as posted: {}", id);
        self.posted_ids.insert(id.to_string(), true);
        self.save();
    }

    /// Check if a video has been posted
    pub fn is_posted(&self, id: &str) -> bool {
        *self.posted_ids.get(id).unwrap_or(&false)
    }
}

impl Default for PostState {
    fn default() -> Self {
        Self {
            posted_ids: HashMap::new(),
        }
    }
}
