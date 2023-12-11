use std::{process::Child, sync::Mutex, time::Duration};

use lazy_static::lazy_static;
use testangel_engine::*;
use thirtyfour::prelude::*;
use thiserror::Error;
use tokio::runtime::{self, Runtime};

const DEFAULT_URI: &str = "data:text/html;base64,PGh0bWw+PGhlYWQ+PHRpdGxlPkJyb3dzZXIgQXV0b21hdGlvbjwvdGl0bGU+PC9oZWFkPjxib2R5IHN0eWxlPSJvdmVyZmxvdzpoaWRkZW47Ij48aDEgc3R5bGU9ImRpc3BsYXk6ZmxleDtqdXN0aWZ5LWNvbnRlbnQ6Y2VudGVyO2FsaWduLWl0ZW1zOmNlbnRlcjtoZWlnaHQ6MTAwJTsiPlRlc3RBbmdlbCBCcm93c2VyIEF1dG9tYXRpb24gc3RhcnRpbmcuLi48L2gxPjwvYm9keT48L2h0bWw+";

struct State {
    rt: Option<Runtime>,
    driver: Option<WebDriver>,
    child_driver: Option<Child>,
    timeout: Duration,
    interval: Duration,
}

impl Default for State {
    fn default() -> Self {
        Self {
            rt: None,
            driver: None,
            child_driver: None,
            timeout: Duration::from_secs(10),
            interval: Duration::from_millis(100),
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        if let Some(child) = &mut self.child_driver {
            child.kill().expect("failed to kill driver child");
        }
    }
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

                use std::{env, process};
                let use_chrome = env::var("TA_BROWSER_USE_CHROME").ok();
                let use_firefox = env::var("TA_BROWSER_USE_FIREFOX").ok();
                let webdriver_port = env::var("TA_BROWSER_WEBDRIVER_PORT").ok();

                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = if let Some(chromedriver_path) = use_chrome {
                    // Try to connect to running chromedriver
                    let port = webdriver_port.unwrap_or("9515".to_string());
                    if let Ok(driver) = rt.block_on(WebDriver::new(&format!("http://localhost:{port}"), DesiredCapabilities::chrome())) {
                        driver
                    } else {
                        // Use chromedriver at path
                        let args = env::var("TA_BROWSER_CHROMEDRIVER_ARGS").unwrap_or(String::new());
                        let browser_args = string_to_args(env::var("TA_BROWSER_CHROME_ARGS").unwrap_or(String::new()));
                        state.child_driver = Some(process::Command::new(chromedriver_path)
                            .args(string_to_args(args))
                            .spawn()
                            .map_err(|e| format!("Failed to start chromedriver: {e}"))?);
                        std::thread::sleep(Duration::from_millis(500));
                        let mut caps = DesiredCapabilities::chrome();
                        for arg in browser_args {
                            let _ = caps.add_chrome_arg(&arg);
                        }
                        rt.block_on(WebDriver::new(&format!("http://localhost:{port}"), caps))?
                    }
                } else if let Some(geckodriver_path) = use_firefox {
                    // Try to connect to running geckodriver
                    let port = webdriver_port.unwrap_or("4444".to_string());
                    if let Ok(driver) = rt.block_on(WebDriver::new(&format!("http://localhost:{port}"), DesiredCapabilities::firefox())) {
                        driver
                    } else {
                        // Use geckodriver at path
                        let args = env::var("TA_BROWSER_GECKODRIVER_ARGS").unwrap_or(String::new());
                        let browser_args = string_to_args(env::var("TA_BROWSER_FIREFOX_ARGS").unwrap_or(String::new()));
                        state.child_driver = Some(process::Command::new(geckodriver_path)
                            .args(string_to_args(args))
                            .spawn()
                            .map_err(|e| format!("Failed to start geckodriver: {e}"))?);
                        // Give it time to start
                        std::thread::sleep(Duration::from_millis(500));
                        let mut caps = DesiredCapabilities::firefox();
                        for arg in browser_args {
                            let _ = caps.add_firefox_arg(&arg);
                        }
                        rt.block_on(WebDriver::new(&format!("http://localhost:{port}"), caps))?
                    }
                } else {
                    // TODO Download a browser and driver
                    Err("This functionality is currently not implemented in the engine. Please set either `TA_BROWSER_USE_CHROME` or `TA_BROWSER_USE_FIREFOX` and try again.")?;
                    unreachable!()
                };

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

        /* WEBDRIVER SESSION */
        .with_instruction(
            Instruction::new("browser-alert-dismiss", "Alert: Dismiss", "Dismiss an alert box."),
            |state: &mut Mutex<State>, _params, _output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                rt.block_on(driver.dismiss_alert())?;
                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-alert-accept", "Alert: Accept", "Accept an alert box."),
            |state: &mut Mutex<State>, _params, _output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                rt.block_on(driver.accept_alert())?;
                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-alert-get-text", "Alert: Get Text", "Get the text contained in an alert box.")
                .with_output("text", "Alert Text", ParameterKind::String),
            |state: &mut Mutex<State>, _params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let text = rt.block_on(driver.get_alert_text())?;
                output.insert("text".to_string(), ParameterValue::String(text));
                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-alert-send-text", "Alert: Send Keys (Type)", "Send keys to an alert box.")
                .with_parameter("keys", "Keys", ParameterKind::String),
            |state: &mut Mutex<State>, params, _output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let keys = params["keys"].value_string();
                rt.block_on(driver.send_alert_text(keys))?;
                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-current-url", "Get Current URL", "Get the current URL.")
                .with_output("url", "URL", ParameterKind::String),
            |state: &mut Mutex<State>, _params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let url = rt.block_on(driver.current_url())?;
                output.insert("url".to_string(), ParameterValue::String(url.to_string()));
                Ok(())
            }
        )
        .with_instruction(
            Instruction::new("browser-execute-javascript", "Execute JavaScript", "Execute arbitrary JavaScript.")
                .with_parameter("script", "JavaScript", ParameterKind::String)
                .with_output("return", "Return Value as JSON String", ParameterKind::String),
            |state: &mut Mutex<State>, params, output, _evidence| {
                let state = state.get_mut().map_err(|_| "Serious error: state mutex was poisoned")?;
                let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
                let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

                let script = params["script"].value_string();
                let ret = rt.block_on(driver.execute(&script, vec![]))?;
                output.insert("return".to_string(), ParameterValue::String(ret.json().to_string()));
                Ok(())
            }
        )
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

                let elem = rt.block_on(driver.query(By::ClassName(&params["class"].value_string()))
                    .wait(state.timeout, state.interval)
                    .first())?;
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

                let elem = rt.block_on(driver.query(By::Css(&params["css"].value_string()))
                    .wait(state.timeout, state.interval)
                    .first())?;
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

                let elem = rt.block_on(driver.query(By::Id(&params["id"].value_string()))
                    .wait(state.timeout, state.interval)
                    .first())?;
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

                let elem = rt.block_on(driver.query(By::LinkText(&params["link-text"].value_string()))
                    .wait(state.timeout, state.interval)
                    .first())?;
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

                let elem = rt.block_on(driver.query(By::Name(&params["name"].value_string()))
                    .wait(state.timeout, state.interval)
                    .first())?;
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

                let elem = rt.block_on(driver.query(By::Tag(&params["tag"].value_string()))
                    .wait(state.timeout, state.interval)
                    .first())?;
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

                let elem = rt.block_on(driver.query(By::XPath(&params["xpath"].value_string()))
                    .wait(state.timeout, state.interval)
                    .first())?;
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

fn string_to_args<S: AsRef<str>>(s: S) -> Vec<String> {
    let mut args = vec![];

    let mut quoted = false;
    let mut escaped = false;
    let mut buffer = String::new();

    for ch in s.as_ref().chars() {
        match ch {
            '"' => {
                if escaped {
                    buffer.push(ch);
                } else {
                    quoted = !quoted;
                }
                escaped = false;
            }
            '\\' => {
                escaped = true;
            }
            ' ' => {
                if quoted {
                    buffer.push(ch);
                } else {
                    args.push(buffer);
                    buffer = String::new();
                }
                escaped = false;
            }
            _ => {
                buffer.push(ch);
                escaped = false;
            }
        }
    }

    if !quoted && !buffer.is_empty() {
        args.push(buffer);
    }

    args
}

#[cfg(test)]
mod tests {
    use crate::string_to_args;

    #[test]
    fn test_string_to_args() {
        assert_eq!(string_to_args(""), Vec::<String>::new());
        assert_eq!(
            string_to_args("arga argb argc"),
            vec!["arga", "argb", "argc"]
        );
        assert_eq!(
            string_to_args(r#"arga "argb" argc"#),
            vec!["arga", "argb", "argc"]
        );
        assert_eq!(
            string_to_args(r#"arga "argb argc""#),
            vec!["arga", "argb argc"]
        );
        assert_eq!(
            string_to_args(r#"arga "argb \"argc""#),
            vec!["arga", "argb \"argc"]
        );
    }
}
