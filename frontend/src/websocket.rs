// frontend/src/websocket.rs

// dependencies
use checkers_common::{ClientMessage, ServerMessage};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, WebSocket, js_sys};
use yew::Callback;

pub struct GameSocket {
    ws: WebSocket,
}

impl GameSocket {
    pub fn connect(on_message: Callback<ServerMessage>) -> Result<Self, JsValue> {
        let window = web_sys::window().unwrap();
        let location = window.location();
        let protocol = if location.protocol().unwrap() == "https:" {
            "wss"
        } else {
            "ws"
        };
        let host = location.host().unwrap();
        let url = format!("{}://{}/api/v1/ws", protocol, host);

        log::info!("Connecting to WebSocket: {}", url);
        let ws = WebSocket::new(&url)?;

        let ws_clone = ws.clone();
        let onopen_callback = Closure::<dyn FnMut()>::new(move || {
            log::info!("WebSocket connected, sending JoinGame");
            let msg = serde_json::to_string(&ClientMessage::JoinGame).unwrap();
            let _ = ws_clone.send_with_str(&msg);
        });
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            log::info!("Raw message received: {:?}", e.data());
            if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                let text: String = text.into();
                match serde_json::from_str::<ServerMessage>(&text) {
                    Ok(msg) => on_message.emit(msg),
                    Err(e) => log::error!("Failed to parse server message: {}", e),
                }
            }
        });
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget(); // Leak to keep alive

        Ok(Self { ws })
    }

    pub fn send(&self, msg: ClientMessage) {
        if let Ok(json) = serde_json::to_string(&msg)
            && let Err(e) = self.ws.send_with_str(&json)
        {
            log::error!("Failed to send message: {:?}", e);
        }
    }
}
