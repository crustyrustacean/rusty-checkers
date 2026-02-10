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
        let window =
            web_sys::window().ok_or_else(|| JsValue::from_str("No window object available"))?;
        let location = window.location();
        let protocol = location
            .protocol()
            .map_err(|_| JsValue::from_str("Failed to get protocol"))?;
        let host = location
            .host()
            .map_err(|_| JsValue::from_str("Failed to get host"))?;

        let scheme = if protocol == "https:" { "wss" } else { "ws" };
        let url = format!("{}://{}/api/v1/ws", scheme, host);

        log::info!("Connecting to WebSocket: {}", url);
        let ws = WebSocket::new(&url)?;

        let ws_clone = ws.clone();
        let onopen_callback = Closure::<dyn FnMut()>::new(move || {
            log::info!("WebSocket connected, sending JoinGame");
            if let Ok(msg) = serde_json::to_string(&ClientMessage::JoinGame) {
                let _ = ws_clone.send_with_str(&msg);
            } else {
                log::error!("Failed to serialize JoinGame");
            }
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
