# Streaker <img alt="Streaker Logo" align="right" width=40 height=40 alt="Screenshot" src="./.github/icons/icon.svg">

> 🔥 GitHub contribution streak & stat tracking menu bar app</p>

[![build](https://github.com/jamieweavis/streaker/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/jamieweavis/streaker/actions)
[![downloads](https://img.shields.io/github/downloads/jamieweavis/streaker/total.svg)](https://github.com/jamieweavis/streaker/releases)
[![version](https://img.shields.io/github/release/jamieweavis/streaker.svg)](https://github.com/jamieweavis/streaker/releases)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/jamieweavis/streaker/blob/main/LICENSE)

<img width="716" alt="Screenshot" src=".github/icons/screenshot.png">

## Install

Download the latest version of Streaker from the **[GitHub releases](https://github.com/jamieweavis/streaker/releases)** page. Or see the **[Building Locally](#building-locally)** section to build it yourself.

## Features

- Menu bar contribution stats
  - Streak
    - Best
    - Current
    - Previous
  - Contributions
    - Most (day)
    - Today
    - Total (year)
- Menu bar icon changes depending on contribution status
  - Empty - you haven't contributed today
  - Filled - you have contributed today
  - Crown - you're currently on your best streak
- Three different menu bar icon themes
  - Flame
  - Tile
  - Octocat
- Reminder notification so you don't lose your streak

### Menu Bar Icons

#### Light Mode

| State       | Flame                                                                                  | Tile                                                                                 | Octocat                                                                                    |
| ----------- | -------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------ |
| Pending     | <img width="20" alt="Flame Empty" src=".github/icons/pending-flame-mac.svg">           | <img width="20" alt="Tile Empty" src=".github/icons/pending-tile-mac.svg">           | <img width="20" alt="Octocat Empty" src=".github/icons/pending-octocat-mac.svg">           |
| Contributed | <img width="20" alt="Flame Contributed" src=".github/icons/contributed-flame-mac.svg"> | <img width="20" alt="Tile Contributed" src=".github/icons/contributed-tile-mac.svg"> | <img width="20" alt="Octocat Contributed" src=".github/icons/contributed-octocat-mac.svg"> |
| Streaking   | <img width="20" alt="Flame Crown" src=".github/icons/streaking-flame-mac.svg">         | <img width="20" alt="Tile Streaking" src=".github/icons/streaking-tile-mac.svg">     | <img width="20" alt="Octocat Streaking" src=".github/icons/streaking-octocat-mac.svg">     |

#### Dark Mode

| State       | Flame                                                                                        | Tile                                                                                       | Octocat                                                                                          |
| ----------- | -------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| Pending     | <img width="20" alt="Flame Empty" src=".github/icons/pending-flame-mac-white.svg">           | <img width="20" alt="Tile Empty" src=".github/icons/pending-tile-mac-white.svg">           | <img width="20" alt="Octocat Empty" src=".github/icons/pending-octocat-mac-white.svg">           |
| Contributed | <img width="20" alt="Flame Contributed" src=".github/icons/contributed-flame-mac-white.svg"> | <img width="20" alt="Tile Contributed" src=".github/icons/contributed-tile-mac-white.svg"> | <img width="20" alt="Octocat Contributed" src=".github/icons/contributed-octocat-mac-white.svg"> |
| Streaking   | <img width="20" alt="Flame Crown" src=".github/icons/streaking-flame-mac-white.svg">         | <img width="20" alt="Tile Streaking" src=".github/icons/streaking-tile-mac-white.svg">     | <img width="20" alt="Octocat Streaking" src=".github/icons/streaking-octocat-mac-white.svg">     |

## Building Locally

The app is built & deployed for all platforms via GitHub Actions automatically, but you can build it locally for your platform with the following steps:

Clone the repository
```bash
git clone https://github.com/jamieweavis/streaker.git

cd streaker
```

Install npm dependencies (recommended to use Node.js v22)
```bash
npm install
```

Package the app for your current platform
```bash
npm run package
```

The packaged app will now be in the `out/` directory

## Development

Run the app in development mode
```bash
npm run start
```

*Only the renderer process currently supports hot reloading, changes in `src/main/` will require a restart*

## Built with

- [TypeScript](https://github.com/microsoft/TypeScript)
- [Electron](https://github.com/electron/electron)
- [Tailwind CSS](https://github.com/tailwindlabs/tailwindcss)
- [Electron Forge](https://github.com/electron/forge)

## Related

- [Streaker CLI](https://github.com/jamieweavis/streaker-cli) - 🔥 GitHub contribution streak & stat tracking command line app
- [Contribution](https://github.com/jamieweavis/contribution) - 🗓 GitHub user contribution graph parser, streak & stat calculator

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
