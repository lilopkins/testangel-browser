#![warn(clippy::pedantic)]

use std::{process::Child, time::Duration};

use testangel_engine::{Evidence, EvidenceContent, engine};
use thirtyfour::prelude::*;
use thiserror::Error;
use tokio::runtime::{self, Runtime};

const DEFAULT_URI: &str = "data:text/html;base64,PGh0bWw+PGhlYWQ+PHRpdGxlPkJyb3dzZXIgQXV0b21hdGlvbjwvdGl0bGU+PC9oZWFkPjxib2R5IHN0eWxlPSJvdmVyZmxvdzpoaWRkZW47Ij48aDEgc3R5bGU9ImRpc3BsYXk6ZmxleDtqdXN0aWZ5LWNvbnRlbnQ6Y2VudGVyO2FsaWduLWl0ZW1zOmNlbnRlcjtoZWlnaHQ6MTAwJTsiPlRlc3RBbmdlbCBCcm93c2VyIEF1dG9tYXRpb24gc3RhcnRpbmcuLi48L2gxPjwvYm9keT48L2h0bWw+";
mod utils;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("The browser robot hasn't been initialised before use.")]
    NotInitialised,
}

engine! {
    /// Work with web sites and browsers.
    #[engine(
        name = "Browser Automation",
        version = env!("CARGO_PKG_VERSION"),
    )]
    struct Browser {
        rt: Option<Runtime>,
        driver: Option<WebDriver>,
        child_driver: Option<Child>,
        timeout: Duration,
        interval: Duration,
    }

    impl Browser {
        /* INITIALISE AND DE-INITIALISE */
        /// Connect to the browser robot.
        #[instruction(
            id = "browser-connect",
            lua_name = "ConnectToBrowser",
            name = "Connect to Browser",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn connect() {
            use std::{env, process};

            state.rt = Some(runtime::Builder::new_current_thread().enable_all().build()?);
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
                    let args = env::var("TA_BROWSER_CHROMEDRIVER_ARGS").unwrap_or_default();
                    let browser_args = string_to_args(env::var("TA_BROWSER_CHROME_ARGS").unwrap_or_default());
                    state.child_driver = Some(process::Command::new(chromedriver_path)
                        .args(string_to_args(args))
                        .spawn()
                        .map_err(|e| format!("Failed to start chromedriver: {e}"))?);
                    std::thread::sleep(Duration::from_millis(500));
                    let mut caps = DesiredCapabilities::chrome();
                    for arg in browser_args {
                        let _ = caps.add_arg(&arg);
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
                    let args = env::var("TA_BROWSER_GECKODRIVER_ARGS").unwrap_or_default();
                    let browser_args = string_to_args(env::var("TA_BROWSER_FIREFOX_ARGS").unwrap_or_default());
                    state.child_driver = Some(process::Command::new(geckodriver_path)
                        .args(string_to_args(args))
                        .spawn()
                        .map_err(|e| format!("Failed to start geckodriver: {e}"))?);
                    // Give it time to start
                    std::thread::sleep(Duration::from_millis(500));
                    let mut caps = DesiredCapabilities::firefox();
                    for arg in browser_args {
                        let _ = caps.add_arg(&arg);
                    }
                    rt.block_on(WebDriver::new(&format!("http://localhost:{port}"), caps))?
                }
            } else {
                // TODO Download a browser and driver
                Err("This functionality is currently not implemented in the engine. Please set either `TA_BROWSER_USE_CHROME` or `TA_BROWSER_USE_FIREFOX` and try again.")?;
                unreachable!()
            };

            // Has to use this strange format to prevent data URLs being mangled.
            rt.block_on(driver.goto(DEFAULT_URI))?;
            state.driver = Some(driver);
        }

        /// Quit the browser robot session.
        #[instruction(
            id = "browser-quit",
            lua_name = "Quit",
            name = "Quit Session",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn quit() {
            let rt = state.rt.take().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.take().ok_or(EngineError::NotInitialised)?;
            rt.block_on(driver.quit())?;
        }

        /* WEBDRIVER SESSION */

        /// Dismiss an alert box.
        #[instruction(
            id = "browser-alert-dismiss",
            lua_name = "AlertDismiss",
            name = "Alert: Dismiss",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn alert_dismiss() {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            rt.block_on(driver.dismiss_alert())?;
        }

        /// Accept an alert box.
        #[instruction(
            id = "browser-alert-accept",
            lua_name = "AlertAccept",
            name = "Alert: Accept",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn alert_accept() {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;

            rt.block_on(driver.accept_alert())?;
        }

        /// Get the text contained in an alert box.
        #[instruction(
            id = "browser-alert-get-text",
            lua_name = "AlertGetText",
            name = "Alert: Get Text",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn alert_get_text() -> #[output(id = "text", name = "Alert Text")] String {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            rt.block_on(driver.get_alert_text())?
        }

        /// Send keys to an alert box.
        #[instruction(
            id = "browser-alert-send-text",
            lua_name = "AlertType",
            name = "Alert: Send Keys (Type)",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn alert_send_text(
            keys: String,
        ) {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            rt.block_on(driver.send_alert_text(keys))?;
        }

        /// Get the current URL.
        #[instruction(
            id = "browser-current-url",
            lua_name = "GetCurrentURL",
            name = "Get Current URL",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn current_url() -> #[output(id = "url", name = "URL")] String {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let url = rt.block_on(driver.current_url())?;
            url.to_string()
        }

        /// Execute arbitrary JavaScript.
        #[instruction(
            id = "browser-execute-javascript",
            lua_name = "ExecuteJavaScript",
            name = "Execute JavaScript",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn execute_javascript(
            #[arg(name = "JavaScript")] script: String,
        ) -> #[output(id = "return", name = "Return Value as JSON String")] String {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let ret = rt.block_on(driver.execute(&script, vec![]))?;
            ret.json().to_string()
        }

        /// Direct the browser to a URL.
        #[instruction(
            id = "browser-goto",
            lua_name = "GoToURL",
            name = "Go to URL",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn goto(
            #[arg(name = "URL")] url: String,
        ) {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            rt.block_on(driver.goto(url))?;
        }

        /* CHROME DEVTOOLS PROTOCOL */

        /// Execute arbitrary JavaScript.
        #[instruction(
            id = "browser-cdp-execute",
            lua_name = "CDPExecute",
            name = "Chrome DevTools: Execute",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn cdp_execute(
            #[arg(name = "Command")] cmd: String,
        ) -> #[output(id = "return", name = "Return Value as JSON String")] String {
            use thirtyfour::extensions::cdp::ChromeDevTools;
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let dev_tools = ChromeDevTools::new(driver.handle.clone());
            let ret = rt.block_on(dev_tools.execute_cdp(&cmd))?;
            serde_json::to_string(&ret).map_err(|_| "Return value couldn't be converted to JSON string")?
        }

        /// Direct the browser to a URL.
        #[instruction(
            id = "browser-cdp-execute-with-params",
            lua_name = "CDPExecuteWithParams",
            name = "Chrome DevTools: Execute with Parameters",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn cdp_execute_with_params(
            #[arg(name = "Command")] cmd: String,
            #[arg(name = "Parameter as JSON String")] params: String
        ) -> #[output(id = "return", name = "Return Value as JSON String")] String {
            use thirtyfour::extensions::cdp::ChromeDevTools;
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let dev_tools = ChromeDevTools::new(driver.handle.clone());
            let ret = rt.block_on(dev_tools.execute_cdp_with_params(&cmd, serde_json::from_str(&params).map_err(|_| "Parameters for CDP are not a valid JSON string")?))?;
            serde_json::to_string(&ret).map_err(|_| "Return value couldn't be converted to JSON string")?
        }

        /* ELEMENT SELECTION */

        /// Select Element By: Class Name
        #[instruction(
            id = "browser-select-by-class-name",
            lua_name = "SelectByClassName",
            name = "Select Element By: Class Name",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn select_by_class_name(
            #[arg(name = "Class Name")] class: String,
        ) -> #[output(id = "element", name = "Element")] String {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = rt.block_on(driver.query(By::ClassName(class))
                .wait(state.timeout, state.interval)
                .first())?;
            utils::serialise_elem(&elem)?
        }

        /// Select Element By: CSS Selector
        #[instruction(
            id = "browser-select-by-css",
            lua_name = "SelectByCSS",
            name = "Select Element By: CSS Selector",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn select_by_css(
            #[arg(name = "CSS Selector")] css: String,
        ) -> #[output(id = "element", name = "Element")] String {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = rt.block_on(driver.query(By::Css(css))
                .wait(state.timeout, state.interval)
                .first())?;
            utils::serialise_elem(&elem)?
        }

        /// Select Element By: ID
        #[instruction(
            id = "browser-select-by-id",
            lua_name = "SelectByID",
            name = "Select Element By: ID",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn select_by_id(
            #[arg(name = "ID")] id: String,
        ) -> #[output(id = "element", name = "Element")] String {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = rt.block_on(driver.query(By::Id(id))
                .wait(state.timeout, state.interval)
                .first())?;
            utils::serialise_elem(&elem)?
        }

        /// Select Element By: Link Text
        #[instruction(
            id = "browser-select-by-link-text",
            lua_name = "SelectByLinkText",
            name = "Select Element By: Link Text",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn select_by_link_text(
            #[arg(id = "link-text", name = "Link Text")] link_text: String,
        ) -> #[output(id = "element", name = "Element")] String {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = rt.block_on(driver.query(By::LinkText(link_text))
                .wait(state.timeout, state.interval)
                .first())?;
            utils::serialise_elem(&elem)?
        }

        /// Select Element By: HTML 'name' attribute
        #[instruction(
            id = "browser-select-by-name",
            lua_name = "SelectByName",
            name = "Select Element By: Name",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn select_by_name(
            #[arg(name = "Name")] name: String,
        ) -> #[output(id = "element", name = "Element")] String {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = rt.block_on(driver.query(By::Name(name))
                .wait(state.timeout, state.interval)
                .first())?;
            utils::serialise_elem(&elem)?
        }

        /// Select Element By: Tag
        #[instruction(
            id = "browser-select-by-tag",
            lua_name = "SelectByTag",
            name = "Select Element By: Tag",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn select_by_tag(
            tag: String,
        ) -> #[output(id = "element", name = "Element")] String {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = rt.block_on(driver.query(By::Tag(tag))
                .wait(state.timeout, state.interval)
                .first())?;
            utils::serialise_elem(&elem)?
        }

        /// Select Element By: XPath
        #[instruction(
            id = "browser-select-by-xpath",
            lua_name = "SelectByXPath",
            name = "Select Element By: XPath",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn select_by_xpath(
            #[arg(name = "XPath")] xpath: String,
        ) -> #[output(id = "element", name = "Element")] String {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = rt.block_on(driver.query(By::XPath(xpath))
                .wait(state.timeout, state.interval)
                .first())?;
            utils::serialise_elem(&elem)?
        }

        /* ELEMENT ACTIONS */
        /// Get attribute
        #[instruction(
            id = "browser-element-attr",
            lua_name = "GetElementAttribute",
            name = "Element: Get Attribute",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn element_attr(
            element: String,
            #[arg(name = "Attribute Name")] name: String,
        ) -> #[output(id = "attr", name = "Attribute Value")] String {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = utils::deserialise_elem(&driver.handle, &element)?;
            let val = rt.block_on(elem.attr(&name))?;
            val.unwrap_or(String::new())
        }

        /// Get class name
        #[instruction(
            id = "browser-element-class-name",
            lua_name = "GetElementClassName",
            name = "Element: Get Class Name",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn element_class_name(
            element: String,
        ) -> #[output(id = "class", name = "Class Name")] String {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = utils::deserialise_elem(&driver.handle, &element)?;
            let val = rt.block_on(elem.class_name())?;
            val.unwrap_or(String::new())
        }

        /// Clear the contents, for example of a text field.
        #[instruction(
            id = "browser-element-clear",
            lua_name = "ClearElement",
            name = "Element: Clear",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn element_clear(
            element: String,
        ) {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = utils::deserialise_elem(&driver.handle, &element)?;
            rt.block_on(elem.clear())?;
        }

        /// Click element
        #[instruction(
            id = "browser-element-click",
            lua_name = "ClickElement",
            name = "Element: Click",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn element_click(
            element: String
        ) {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = utils::deserialise_elem(&driver.handle, &element)?;
            rt.block_on(elem.click())?;
        }

        /// Get CSS value
        #[instruction(
            id = "browser-element-css-value",
            lua_name = "GetElementCSSValue",
            name = "Element: Get CSS Value",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn element_css_value(
            element: String,
            #[arg(name = "CSS Property")] name: String,
        ) -> #[output(id = "value", name = "value")] String {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = utils::deserialise_elem(&driver.handle, &element)?;
            rt.block_on(elem.css_value(&name))?
        }

        /// Focus this element using JavaScript
        #[instruction(
            id = "browser-element-focus",
            lua_name = "FocusElement",
            name = "Element: Focus",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn element_focus(
            element: String,
        ) {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = utils::deserialise_elem(&driver.handle, &element)?;
            rt.block_on(elem.focus())?;
        }

        /// Get element ID
        #[instruction(
            id = "browser-element-id",
            lua_name = "GetElementID",
            name = "Element: Get ID",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn element_id(
            element: String,
        ) -> #[output(id = "id", name = "Element ID")] String {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = utils::deserialise_elem(&driver.handle, &element)?;
            let val = rt.block_on(elem.id())?;
            val.unwrap_or(String::new())
        }

        /// Get the HTML within this element's nodes
        #[instruction(
            id = "browser-element-inner-html",
            lua_name = "GetElementInnerHTML",
            name = "Element: Get Inner HTML",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn element_inner_html(
            element: String,
        ) -> #[output(id = "html", name = "Inner HTML")] String {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = utils::deserialise_elem(&driver.handle, &element)?;
            rt.block_on(elem.inner_html())?
        }

        /// Return is the element is clickable (visible and enabled).
        #[instruction(
            id = "browser-element-is-clickable",
            lua_name = "IsElementClickable",
            name = "Element: Is Clickable",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn element_is_clickable(
            element: String,
        ) -> #[output(id = "clickable", name = "Clickable")] bool {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = utils::deserialise_elem(&driver.handle, &element)?;
            rt.block_on(elem.is_clickable())?
        }

        /// Return is the element is displayed.
        #[instruction(
            id = "browser-element-is-displayed",
            lua_name = "IsElementDisplayed",
            name = "Element: Is Displayed",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn element_is_displayed(
            element: String,
        ) -> #[output(id = "displayed", name = "Displayed")] bool {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = utils::deserialise_elem(&driver.handle, &element)?;
            rt.block_on(elem.is_displayed())?
        }

        /// Return is the element is enabled.
        #[instruction(
            id = "browser-element-is-enabled",
            lua_name = "IsElementEnabled",
            name = "Element: Is Enabled",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn element_is_enabled(
            element: String,
        ) -> #[output(id = "enabled", name = "Enabled")] bool {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = utils::deserialise_elem(&driver.handle, &element)?;
            rt.block_on(elem.is_enabled())?
        }

        /// Return is the element is selected.
        #[instruction(
            id = "browser-element-is-selected",
            lua_name = "IsElementSelected",
            name = "Element: Is Selected",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn element_is_selected(
            element: String,
        ) -> #[output(id = "selected", name = "Selected")] bool {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = utils::deserialise_elem(&driver.handle, &element)?;
            rt.block_on(elem.is_selected())?
        }

        /// Get the HTML within this element's nodes
        #[instruction(
            id = "browser-element-outer-html",
            lua_name = "GetElementOuterHTML",
            name = "Element: Get Outer HTML",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn element_outer_html(
            element: String,
        ) -> #[output(id = "html", name = "Outer HTML")] String {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = utils::deserialise_elem(&driver.handle, &element)?;
            rt.block_on(elem.outer_html())?
        }

        /// Screenshot an element as evidence
        #[instruction(
            id = "browser-element-screenshot",
            lua_name = "ScreenshotElementAsEvidence",
            name = "Element: Screenshot as Evidence",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn element_screenshot(
            element: String,
            label: String,
        ) {
            use base64::{Engine as _, engine::general_purpose};

            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = utils::deserialise_elem(&driver.handle, &element)?;

            let png_data = rt.block_on(elem.screenshot_as_png())?;
            let png_base64 = general_purpose::STANDARD.encode(png_data);
            evidence.push(Evidence { label, content: EvidenceContent::ImageAsPngBase64(png_base64) });
        }

        /// Scroll this element into view using JavaScript
        #[instruction(
            id = "browser-element-scroll-into-view",
            lua_name = "ScrollElementIntoView",
            name = "Element: Scroll into View",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn element_scroll_into_view(
            element: String,
        ) {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = utils::deserialise_elem(&driver.handle, &element)?;
            rt.block_on(elem.scroll_into_view())?;
        }

        /// Send keys (type) to this element. For special keys, see: hpkns.uk/takeys
        #[instruction(
            id = "browser-element-send-keys",
            lua_name = "ElementType",
            name = "Element: Send Keys (Type)",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn element_send_keys(
            element: String,
            keys: String,
        ) {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = utils::deserialise_elem(&driver.handle, &element)?;
            rt.block_on(elem.send_keys(keys))?;
        }

        /// Get the text within this element's nodes
        #[instruction(
            id = "browser-element-text",
            lua_name = "GetElementText",
            name = "Element: Get Text",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn element_text(
            element: String,
        ) -> #[output(id = "text", name = "Text")] String {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = utils::deserialise_elem(&driver.handle, &element)?;
            rt.block_on(elem.text())?
        }

        /// Get the value of this element
        #[instruction(
            id = "browser-element-value",
            lua_name = "GetElementValue",
            name = "Element: Get Value",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn element_value(
            element: String,
        ) -> #[output(id = "value", name = "Value")] String {
            let rt = state.rt.as_ref().ok_or(EngineError::NotInitialised)?;
            let driver = state.driver.as_ref().ok_or(EngineError::NotInitialised)?;
            let elem = utils::deserialise_elem(&driver.handle, &element)?;
            let val = rt.block_on(elem.value())?;
            val.unwrap_or(String::new())
        }
    }
}

impl Default for Browser {
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

impl Drop for Browser {
    fn drop(&mut self) {
        if let Some(child) = &mut self.child_driver {
            child.kill().expect("failed to kill driver child");
        }
    }
}

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
