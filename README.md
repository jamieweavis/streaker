# Streaker <img src="./screenshots/icon.svg" align="right">

> 🐙 GitHub contribution streak & stat tracking menu bar app

[![Build](https://img.shields.io/travis/jamieweavis/streaker.svg)](https://travis-ci.org/jamieweavis/streaker)
[![Downloads](https://img.shields.io/github/downloads/jamieweavis/streaker/total.svg)](https://github.com/jamieweavis/streaker/releases)
[![Version](https://img.shields.io/github/release/jamieweavis/streaker.svg)](https://github.com/jamieweavis/streaker/releases)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/jamieweavis/streaker/blob/master/LICENSE.md)
[![Style](https://img.shields.io/badge/code_style-prettier-ff69b4.svg)](https://prettier.io/)

## Contents

- [Installation](#installation)
- [Features](#features)
- [Screenshots](#screenshots)
- [Menu Bar Icons](#menu-bar-icons)
- [Related](#related)

## Installation

Download the latest version of Streaker from the **[GitHub releases](https://github.com/jamieweavis/streaker/releases)** page. (currently macOS & Windows only)

## Features

- Current streak, best streak, current contributions, best contributions & total contributions stats
- Launch at login
- Notification when you haven't contributed today

## Screenshots

| Menu Bar                                                                     | Preferences                                                                        |
| ---------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| <img src="./screenshots/menu-bar.png" width="257" alt="Menu Bar Screenshot"> | <img src="./screenshots/preferences.png" width="344" alt="Preferences Screenshot"> |

## Menu Bar Icons

Streaker's menu bar icon changes depending on your current daily contribution status:

|                                    | Status  | Description                                           |
| ---------------------------------- | ------- | ----------------------------------------------------- |
| ![Done](./screenshots/done.svg)    | Done    | You've contributed today                              |
| ![Todo](./screenshots/todo.svg)    | Todo    | You haven't contributed today                         |
| ![Loading](./screenshots/load.svg) | Loading | Your contribution data is being requested from GitHub |
| ![Fail](./screenshots/fail.svg)    | Failed  | Your contribution data request failed                 |

## Related

- [streaker-cli](https://github.com/jamieweavis/streaker-cli) - 🐙 GitHub contribution streak & stat tracking CLI app
- [contribution](https://github.com/jamieweavis/contribution) - 🗓 GitHub contribution streak & stat fetcher with zero dependencies

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.
