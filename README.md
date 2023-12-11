# TestAngel Browser Automation Engine

## Environment Configuration

Environment Variable | Purpose
---------------------|--------
`TA_BROWSER_USE_CHROME` | Specify a path to `chromedriver` to use.
`TA_BROWSER_CHROME_ARGS` | Specify additional arguments to pass to `chrome`.
`TA_BROWSER_CHROMEDRIVER_ARGS` | Specify additional arguments to pass to the `chromedriver`.
`TA_BROWSER_USE_FIREFOX` | Specify a path to `geckodriver` to use.
`TA_BROWSER_FIREFOX_ARGS` | Specify additional arguments to pass to `firefox`.
`TA_BROWSER_GECKODRIVER_ARGS` | Specify additional arguments to pass to the `geckodriver`.
`TA_BROWSER_WEBDRIVER_PORT` | Specify a port to use for the webdriver instead of the default.

If no driver is specified manually, a driver will be downloaded automatically and ran from a temporary directory.
