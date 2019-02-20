<p align="center"><img src="./build/icon.png" height="64" alt="Streaker Logo"></p>
<h3 align="center">Streaker</h3>
<p align="center">🐙 GitHub contribution streak tracking menubar app<p>
<p align="center">
    <a href="https://github.com/jamieweavis/streaker/releases"><img src="https://img.shields.io/github/downloads/jamieweavis/streaker/total.svg" alt="GitHub Downloads"></a>
    <a href="https://github.com/jamieweavis/streaker/releases"><img src="https://img.shields.io/github/release/jamieweavis/streaker.svg" alt="Current Release Version"></a>
    <a href="https://github.com/jamieweavis/readme-template/blob/master/LICENSE.md"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
    <a href="https://github.com/prettier/prettier"><img src="https://img.shields.io/badge/code_style-prettier-ff69b4.svg" alt="Prettier Code Style"></a>
</p>

## Installation

Download the latest version of Streaker from the **[GitHub releases](https://github.com/jamieweavis/streaker/releases)** page. (currently macOS & Windows only)

## Features

- Current streak, best streak, total contribution & best day stats
- Launch at login
- Notification when you haven't contributed today

## Screenshots

### Menu Bar

<img src="./screenshots/menu-bar.png" width="280" alt="Menu Bar Screenshot">

### Preferences Window

<img src="./screenshots/preferences-window.png" width="380" alt="Preferences Screenshot">

## Menu Bar Icons

Streaker's menu bar icon is updated depending on your current contribution status:

|                                                              | Status  | Description                                           |
| ------------------------------------------------------------ | ------- | ----------------------------------------------------- |
| <img src="./src/icons/macos/doneTemplate@2x.png" width="16"> | Done    | You've contributed today                              |
| <img src="./src/icons/macos/todoTemplate@2x.png" width="16"> | Todo    | You haven't contributed today                         |
| <img src="./src/icons/macos/loadTemplate@2x.png" width="16"> | Loading | Your contribution data is being requested from GitHub |
| <img src="./src/icons/macos/failTemplate@2x.png" width="16"> | Failed  | Your contribution data request failed                 |

## Related

- [streaker-cli](https://github.com/jamieweavis/streaker-cli) - 🐙 GitHub contribution streak fetching CLI app
- [contribution](https://github.com/jamieweavis/contribution) - 🗓 GitHub contribution count & streak fetcher with zero dependencies

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.
