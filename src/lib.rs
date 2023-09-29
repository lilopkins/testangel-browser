use std::{
    io::{BufReader, BufWriter, Cursor},
    sync::Mutex,
};

use lazy_static::lazy_static;
use testangel_engine::*;
use thirtyfour::prelude::*;
use thiserror::Error;
use tokio::runtime::{self, Runtime};

const DEFAULT_URI: &str = "data:text/html;base64,PGh0bWw+PGhlYWQ+PHRpdGxlPkJyb3dzZXIgQXV0b21hdGlvbjwvdGl0bGU+PC9oZWFkPjxib2R5IHN0eWxlPSJvdmVyZmxvdzpoaWRkZW47Ij48aDEgc3R5bGU9ImRpc3BsYXk6ZmxleDtqdXN0aWZ5LWNvbnRlbnQ6Y2VudGVyO2FsaWduLWl0ZW1zOmNlbnRlcjtoZWlnaHQ6MTAwJTsiPlRlc3RBbmdlbCBCcm93c2VyIEF1dG9tYXRpb24gc3RhcnRpbmcuLi48L2gxPjwvYm9keT48L2h0bWw+";

#[derive(Default)]
struct State {
    rt: Option<Runtime>,
    driver: Option<WebDriver>,
}

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("The browser robot hasn't been initialised before use.")]
    NotInitialised,
}

lazy_static! {
    static ref ENGINE: Mutex<Engine<'static, Mutex<State>>> = Mutex::new(
        Engine::new("Browser Automation", env!("CARGO_PKG_VERSION"))
        /* INITIALISE AND DE-INITIALISE */
        .with_instruction(
            Instruction::new("browser-connect", "Connect to Browser", "Connect to the browser robot."),
            |state: &mut Mutex<State>, _params, _output, _evidence| {
                // Initialising the state initialises the runtime and starts the webdriver.
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                state.rt = Some(runtime::Builder::new_current_thread().enable_all().build()?);

                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = rt.block_on(WebDriver::new("http://localhost:4444", DesiredCapabilities::firefox()))?;
                rt.block_on(driver.goto(DEFAULT_URI))?;
                state.driver = Some(driver);

                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-quit", "Quit Session", "Quit the browser robot session."),
            |state: &mut Mutex<State>, _params, _output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.take().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.take().ok_or(EngineError::NotInitialised)?;

                rt.block_on(driver.quit())?;
                Ok(())
            }
        )

        /* NAVIGATION */
        .with_instruction(
            Instruction::new("browser-goto", "Go to URL", "Direct the browser to a URL.")
                .with_parameter("url", "URL", ParameterKind::String),
            |state: &mut Mutex<State>, params, _output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                rt.block_on(driver.goto(params["url"].value_string()))?;
                Ok(())
            }
        )

        /* ELEMENT SELECTION */
        .with_instruction(
            Instruction::new("browser-select-by-class-name", "Select Element By: Class Name", "Select Element By: Class Name")
                .with_parameter("class", "Class Name", ParameterKind::String)
                .with_output("element", "Element", ParameterKind::String),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let elem = rt.block_on(driver.find(By::ClassName(&params["class"].value_string())))?;
                output.insert("element".to_string(), ParameterValue::String(elem.to_json()?.to_string()));
                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-select-by-css", "Select Element By: CSS Selector", "Select Element By: CSS Selector")
                .with_parameter("css", "CSS Selector", ParameterKind::String)
                .with_output("element", "Element", ParameterKind::String),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let elem = rt.block_on(driver.find(By::Css(&params["css"].value_string())))?;
                output.insert("element".to_string(), ParameterValue::String(elem.to_json()?.to_string()));
                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-select-by-id", "Select Element By: ID", "Select Element By: ID")
                .with_parameter("id", "ID", ParameterKind::String)
                .with_output("element", "Element", ParameterKind::String),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let elem = rt.block_on(driver.find(By::Id(&params["id"].value_string())))?;
                output.insert("element".to_string(), ParameterValue::String(elem.to_json()?.to_string()));
                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-select-by-link-text", "Select Element By: Link Text", "Select Element By: Link Text")
                .with_parameter("link-text", "Link Text", ParameterKind::String)
                .with_output("element", "Element", ParameterKind::String),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let elem = rt.block_on(driver.find(By::LinkText(&params["link-text"].value_string())))?;
                output.insert("element".to_string(), ParameterValue::String(elem.to_json()?.to_string()));
                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-select-by-name", "Select Element By: Name", "Select Element By: HTML 'name' attribute")
                .with_parameter("name", "Name", ParameterKind::String)
                .with_output("element", "Element", ParameterKind::String),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let elem = rt.block_on(driver.find(By::Name(&params["name"].value_string())))?;
                output.insert("element".to_string(), ParameterValue::String(elem.to_json()?.to_string()));
                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-select-by-tag", "Select Element By: Tag", "Select Element By: Tag")
                .with_parameter("tag", "Tag", ParameterKind::String)
                .with_output("element", "Element", ParameterKind::String),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let elem = rt.block_on(driver.find(By::Tag(&params["tag"].value_string())))?;
                output.insert("element".to_string(), ParameterValue::String(elem.to_json()?.to_string()));
                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-select-by-xpath", "Select Element By: XPath", "Select Element By: XPath")
                .with_parameter("xpath", "XPath", ParameterKind::String)
                .with_output("element", "Element", ParameterKind::String),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let elem = rt.block_on(driver.find(By::XPath(&params["xpath"].value_string())))?;
                output.insert("element".to_string(), ParameterValue::String(elem.to_json()?.to_string()));
                Ok(())
            }
        )

        /* ELEMENT ACTIONS */
        .with_instruction(
            Instruction::new("browser-element-attr", "Element: Get Attribute", "Get attribute")
                .with_parameter("element", "Element", ParameterKind::String)
                .with_parameter("name", "Attribute Name", ParameterKind::String)
                .with_output("attr", "Attribute Value", ParameterKind::String),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let json_elem = serde_json::from_str(&params["element"].value_string()).map_err(|e| format!("Invalid element parameter: {e}"))?;
                let elem = WebElement::from_json(json_elem, driver.handle.clone()).map_err(|e| format!("Invalid element: {e}"))?;

                let val = rt.block_on(elem.attr(&params["name"].value_string()))?;
                output.insert("attr".to_string(), ParameterValue::String(val.unwrap_or(String::new())));

                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-element-class-name", "Element: Get Class Name", "Get class name")
                .with_parameter("element", "Element", ParameterKind::String)
                .with_output("class", "Class Name", ParameterKind::String),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let json_elem = serde_json::from_str(&params["element"].value_string()).map_err(|e| format!("Invalid element parameter: {e}"))?;
                let elem = WebElement::from_json(json_elem, driver.handle.clone()).map_err(|e| format!("Invalid element: {e}"))?;

                let val = rt.block_on(elem.class_name())?;
                output.insert("class".to_string(), ParameterValue::String(val.unwrap_or(String::new())));

                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-element-clear", "Element: Clear", "Clear the contents, for example of a text field.")
                .with_parameter("element", "Element", ParameterKind::String),
            |state: &mut Mutex<State>, params, _output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let json_elem = serde_json::from_str(&params["element"].value_string()).map_err(|e| format!("Invalid element parameter: {e}"))?;
                let elem = WebElement::from_json(json_elem, driver.handle.clone()).map_err(|e| format!("Invalid element: {e}"))?;

                rt.block_on(elem.clear())?;
                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-element-click", "Element: Click", "Click element")
                .with_parameter("element", "Element", ParameterKind::String),
            |state: &mut Mutex<State>, params, _output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let json_elem = serde_json::from_str(&params["element"].value_string()).map_err(|e| format!("Invalid element parameter: {e}"))?;
                let elem = WebElement::from_json(json_elem, driver.handle.clone()).map_err(|e| format!("Invalid element: {e}"))?;

                rt.block_on(elem.click())?;
                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-element-css-value", "Element: Get CSS Value", "Get CSS value")
                .with_parameter("element", "Element", ParameterKind::String)
                .with_parameter("name", "CSS property", ParameterKind::String)
                .with_output("value", "Value", ParameterKind::String),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let json_elem = serde_json::from_str(&params["element"].value_string()).map_err(|e| format!("Invalid element parameter: {e}"))?;
                let elem = WebElement::from_json(json_elem, driver.handle.clone()).map_err(|e| format!("Invalid element: {e}"))?;

                let val = rt.block_on(elem.css_value(&params["name"].value_string()))?;
                output.insert("value".to_string(), ParameterValue::String(val));

                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-element-focus", "Element: Focus", "Focus this element using JavaScript")
                .with_parameter("element", "Element", ParameterKind::String),
            |state: &mut Mutex<State>, params, _output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let json_elem = serde_json::from_str(&params["element"].value_string()).map_err(|e| format!("Invalid element parameter: {e}"))?;
                let elem = WebElement::from_json(json_elem, driver.handle.clone()).map_err(|e| format!("Invalid element: {e}"))?;

                rt.block_on(elem.focus())?;
                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-element-id", "Element: Get ID", "Get element ID")
                .with_parameter("element", "Element", ParameterKind::String)
                .with_output("id", "Element ID", ParameterKind::String),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let json_elem = serde_json::from_str(&params["element"].value_string()).map_err(|e| format!("Invalid element parameter: {e}"))?;
                let elem = WebElement::from_json(json_elem, driver.handle.clone()).map_err(|e| format!("Invalid element: {e}"))?;

                let val = rt.block_on(elem.id())?;
                output.insert("id".to_string(), ParameterValue::String(val.unwrap_or(String::new())));

                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-element-inner-html", "Element: Get Inner HTML", "Get the HTML within this element's nodes")
                .with_parameter("element", "Element", ParameterKind::String)
                .with_output("html", "Inner HTML", ParameterKind::String),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let json_elem = serde_json::from_str(&params["element"].value_string()).map_err(|e| format!("Invalid element parameter: {e}"))?;
                let elem = WebElement::from_json(json_elem, driver.handle.clone()).map_err(|e| format!("Invalid element: {e}"))?;

                let val = rt.block_on(elem.inner_html())?;
                output.insert("html".to_string(), ParameterValue::String(val));

                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-element-is-clickable", "Element: Is Clickable", "Return is the element is clickable (visible and enabled).")
                .with_parameter("element", "Element", ParameterKind::String)
                .with_output("clickable", "Clickable", ParameterKind::Boolean),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let json_elem = serde_json::from_str(&params["element"].value_string()).map_err(|e| format!("Invalid element parameter: {e}"))?;
                let elem = WebElement::from_json(json_elem, driver.handle.clone()).map_err(|e| format!("Invalid element: {e}"))?;

                let val = rt.block_on(elem.is_clickable())?;
                output.insert("clickable".to_string(), ParameterValue::Boolean(val));

                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-element-is-displayed", "Element: Is Displayed", "Return is the element is displayed.")
                .with_parameter("element", "Element", ParameterKind::String)
                .with_output("displayed", "Displayed", ParameterKind::Boolean),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let json_elem = serde_json::from_str(&params["element"].value_string()).map_err(|e| format!("Invalid element parameter: {e}"))?;
                let elem = WebElement::from_json(json_elem, driver.handle.clone()).map_err(|e| format!("Invalid element: {e}"))?;

                let val = rt.block_on(elem.is_displayed())?;
                output.insert("displayed".to_string(), ParameterValue::Boolean(val));

                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-element-is-enabled", "Element: Is Enabled", "Return is the element is enabled.")
                .with_parameter("element", "Element", ParameterKind::String)
                .with_output("enabled", "Enabled", ParameterKind::Boolean),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let json_elem = serde_json::from_str(&params["element"].value_string()).map_err(|e| format!("Invalid element parameter: {e}"))?;
                let elem = WebElement::from_json(json_elem, driver.handle.clone()).map_err(|e| format!("Invalid element: {e}"))?;

                let val = rt.block_on(elem.is_enabled())?;
                output.insert("enabled".to_string(), ParameterValue::Boolean(val));

                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-element-is-selected", "Element: Is Selected", "Return is the element is selected.")
                .with_parameter("element", "Element", ParameterKind::String)
                .with_output("selected", "Selected", ParameterKind::Boolean),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let json_elem = serde_json::from_str(&params["element"].value_string()).map_err(|e| format!("Invalid element parameter: {e}"))?;
                let elem = WebElement::from_json(json_elem, driver.handle.clone()).map_err(|e| format!("Invalid element: {e}"))?;

                let val = rt.block_on(elem.is_selected())?;
                output.insert("selected".to_string(), ParameterValue::Boolean(val));

                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-element-outer-html", "Element: Get Outer HTML", "Get the HTML within this element's nodes")
                .with_parameter("element", "Element", ParameterKind::String)
                .with_output("html", "Outer HTML", ParameterKind::String),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let json_elem = serde_json::from_str(&params["element"].value_string()).map_err(|e| format!("Invalid element parameter: {e}"))?;
                let elem = WebElement::from_json(json_elem, driver.handle.clone()).map_err(|e| format!("Invalid element: {e}"))?;

                let val = rt.block_on(elem.outer_html())?;
                output.insert("html".to_string(), ParameterValue::String(val));

                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-element-screenshot", "Element: Screenshot as Evidence", "Screenshot an element as evidence")
                .with_parameter("element", "Element", ParameterKind::String)
                .with_parameter("label", "Label", ParameterKind::String),
            |state: &mut Mutex<State>, params, _output, evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let json_elem = serde_json::from_str(&params["element"].value_string()).map_err(|e| format!("Invalid element parameter: {e}"))?;
                let elem = WebElement::from_json(json_elem, driver.handle.clone()).map_err(|e| format!("Invalid element: {e}"))?;
                let png_data = rt.block_on(elem.screenshot_as_png())?;

                // remove alpha channel
                let img = image::io::Reader::new(BufReader::new(Cursor::new(png_data))).with_guessed_format()?.decode()?.into_rgb8();
                let mut png_data = vec![];
                img.write_to(&mut BufWriter::new(Cursor::new(&mut png_data)), image::ImageOutputFormat::Png)?;

                use base64::{Engine as _, engine::general_purpose};
                let png_base64 = general_purpose::STANDARD.encode(png_data);
                evidence.push(Evidence { label: params["label"].value_string(), content: EvidenceContent::ImageAsPngBase64(png_base64) });

                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-element-scroll-into-view", "Element: Scroll into View", "Scroll this element into view using JavaScript")
                .with_parameter("element", "Element", ParameterKind::String),
            |state: &mut Mutex<State>, params, _output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let json_elem = serde_json::from_str(&params["element"].value_string()).map_err(|e| format!("Invalid element parameter: {e}"))?;
                let elem = WebElement::from_json(json_elem, driver.handle.clone()).map_err(|e| format!("Invalid element: {e}"))?;

                rt.block_on(elem.scroll_into_view())?;
                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-element-send-keys", "Element: Send Keys (Type)", "Send keys (type) to this element. For special keys, see: hpkns.uk/takeys")
                .with_parameter("element", "Element", ParameterKind::String)
                .with_parameter("keys", "Keys", ParameterKind::String),
            |state: &mut Mutex<State>, params, _output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let json_elem = serde_json::from_str(&params["element"].value_string()).map_err(|e| format!("Invalid element parameter: {e}"))?;
                let elem = WebElement::from_json(json_elem, driver.handle.clone()).map_err(|e| format!("Invalid element: {e}"))?;

                rt.block_on(elem.send_keys(params["keys"].value_string()))?;
                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-element-text", "Element: Get Text", "Get the text within this element's nodes")
                .with_parameter("element", "Element", ParameterKind::String)
                .with_output("text", "Text", ParameterKind::String),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let json_elem = serde_json::from_str(&params["element"].value_string()).map_err(|e| format!("Invalid element parameter: {e}"))?;
                let elem = WebElement::from_json(json_elem, driver.handle.clone()).map_err(|e| format!("Invalid element: {e}"))?;

                let val = rt.block_on(elem.text())?;
                output.insert("text".to_string(), ParameterValue::String(val));

                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-element-value", "Element: Get Value", "Get the value of this element")
                .with_parameter("element", "Element", ParameterKind::String)
                .with_output("value", "Value", ParameterKind::String),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let json_elem = serde_json::from_str(&params["element"].value_string()).map_err(|e| format!("Invalid element parameter: {e}"))?;
                let elem = WebElement::from_json(json_elem, driver.handle.clone()).map_err(|e| format!("Invalid element: {e}"))?;

                let val = rt.block_on(elem.value())?;
                output.insert("value".to_string(), ParameterValue::String(val.unwrap_or(String::new())));

                Ok(())
            }
        )
    );
}

expose_engine!(ENGINE);
