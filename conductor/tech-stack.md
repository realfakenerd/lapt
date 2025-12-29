# Technology Stack - lapt

## Core Language and Runtime

- **Rust (Edition 2024):** The primary programming language, ensuring memory safety and high performance.
- **Tokio:** The asynchronous runtime used for handling concurrent tasks, including TUI events and backend package management operations.

## User Interface (TUI)

- **Ratatui:** The core library for building the terminal user interface components and layouts.
- **Crossterm:** The cross-platform terminal backend for handling raw mode, input events, and screen manipulation.
- **Tachyonfx:** A library for adding smooth animations and visual effects to the Ratatui-based UI.

## Backend and Integration

- **PackageKit (via zbus):** The primary system abstraction layer for performing distribution-agnostic package operations.
- **Async Command Wrappers:** Use of `tokio::process::Command` for executing and managing external terminal commands asynchronously when native library integration is not available or preferred.

## Utilities and Error Handling

- **Anyhow:** For flexible and idiomatic error handling across the application.
- **Fuzzy-matcher:** Provides the Skim-based fuzzy search functionality for filtering package lists.
- **Strum:** For working with enums more effectively (e.g., deriving display and iteration traits).
- **Futures-util:** Provides additional utility functions for working with asynchronous streams and futures.
