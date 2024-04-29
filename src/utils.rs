use std::sync::Arc;

use thirtyfour::{error::WebDriverResult, session::handle::SessionHandle, WebElement};

pub fn serialise_elem(elem: WebElement) -> WebDriverResult<String> {
    Ok(elem.to_json()?.to_string())
}

pub fn deserialise_elem<S: AsRef<str>>(handle: Arc<SessionHandle>, s: S) -> Result<WebElement, String> {
    let mut s = s.as_ref();
    let json_elem = serde_json::from_str(s).map_err(|e| format!("Invalid element parameter: {e}"))?;
    WebElement::from_json(json_elem, handle.clone()).map_err(|e| format!("Invalid element: {e}"))
}
