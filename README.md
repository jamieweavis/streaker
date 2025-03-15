# Streaker <img alt="Streaker Logo" align="right" width=40 height=40 alt="Screenshot" src="./.github/icon.svg">

> 🔥 GitHub contribution streak & stat tracking menu bar app</p>

[![build](https://github.com/jamieweavis/streaker/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/jamieweavis/streaker/actions)
[![downloads](https://img.shields.io/github/downloads/jamieweavis/streaker/total.svg)](https://github.com/jamieweavis/streaker/releases)
[![version](https://img.shields.io/github/release/jamieweavis/streaker.svg)](https://github.com/jamieweavis/streaker/releases)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/jamieweavis/streaker/blob/main/LICENSE)

<img width="716" alt="Screenshot" src=".github/screenshot.png">

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
  - Crown / Star - you're currently on your best streak
- Three different menu bar icon themes
  - Flame
  - Tile
  - Octocat
- Reminder notification so you don't lose your streak

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

The packaged app will be in the `out/` directory

## Built with

- [TypeScript](https://github.com/microsoft/TypeScript)
- [Electron](https://github.com/electron/electron)
- [Tailwind CSS](https://github.com/tailwindlabs/tailwindcss)
- [Electron Forge](https://github.com/electron/forge)

## Related

- [Streaker CLI](https://github.com/jamieweavis/streaker-cli) - 🔥 GitHub contribution streak & stat tracking CLI app
- [Contribution](https://github.com/jamieweavis/contribution) - 🗓 GitHub contribution streak & stat fetcher with zero dependencies
