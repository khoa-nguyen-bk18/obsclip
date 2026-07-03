use serde::Serialize;
use std::sync::Mutex;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrHealth {
    pub message: Option<String>,
    pub fix: Option<String>,
}

#[derive(Debug)]
pub struct OcrHealthState(pub Mutex<OcrHealth>);

impl OcrHealthState {
    pub fn new() -> Self {
        Self(Mutex::new(OcrHealth::default()))
    }

    pub fn set_error(&self, message: impl Into<String>, fix: impl Into<String>) {
        let mut health = self.0.lock().unwrap();
        health.message = Some(message.into());
        health.fix = Some(fix.into());
    }

    pub fn clear(&self) {
        let mut health = self.0.lock().unwrap();
        *health = OcrHealth::default();
    }

    pub fn snapshot(&self) -> OcrHealth {
        self.0.lock().unwrap().clone()
    }
}
