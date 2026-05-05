use std::sync::{Arc, Mutex};
use bevy::prelude::Resource;

#[derive(Resource, Clone)]
pub struct SharedState {
    pub selected: Arc<Mutex<Option<(i32, i32, i32)>>>,
    pub requested_xyz: Arc<Mutex<(u32, u32, u32)>>,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            selected: Arc::new(Mutex::new(None)),
            requested_xyz: Arc::new(Mutex::new((5, 5, 5))),
        }
    }
}