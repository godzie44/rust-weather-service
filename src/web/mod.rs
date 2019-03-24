use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;

pub mod action;
mod test;

pub struct AppState {
    pub config: Arc<Mutex<HashMap<String, String>>>,
}