# Plan: Transition to APT Command Wrappers (lazygit-style)

## Phase 1: Deprecation & New Backend Architecture [checkpoint: 44a4dcb]
Focus: Removing PackageKit and establishing the foundation for the new APT wrapper-based backend.

- [x] Task: Remove `PackageKit` dependencies from `Cargo.toml` and clean up imports. [90a12ee]
    - [ ] Sub-task: Implement Feature (Remove `packagekit-zbus` and related crates).
- [x] Task: Refactor `backend.rs` to use `AptBackend` as the primary engine. [790448c]
    - [ ] Sub-task: Write Tests (Ensure `BackendCommand` and `BackendEvent` still meet application needs).
    - [ ] Sub-task: Implement Feature (Redefine `PackageKitBackend` as `AptBackend` or similar).
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Deprecation & New Backend Architecture' (Protocol in workflow.md)

## Phase 2: Core Data Fetching (Listing & Searching)
Focus: Implementing robust parsing for standard APT list and search commands.

- [ ] Task: Implement `apt list` parsing for installed and upgradable packages.
    - [ ] Sub-task: Write Tests (Test with sample `apt list` outputs).
    - [ ] Sub-task: Implement Feature (Create the parser in `src/apt.rs`).
- [ ] Task: Implement `apt search` parsing logic.
    - [ ] Sub-task: Write Tests (Verify search result extraction).
    - [ ] Sub-task: Implement Feature (Add search wrapper to `src/apt.rs`).
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Core Data Fetching (Listing & Searching)' (Protocol in workflow.md)

## Phase 3: System Altering Operations & Confirmation
Focus: Implementing package management commands and the hybrid confirmation logic.

- [ ] Task: Implement `install` and `remove` command wrappers.
    - [ ] Sub-task: Write Tests (Mock command execution and verify flags).
    - [ ] Sub-task: Implement Feature (Execute `apt-get install/remove -y` asynchronously).
- [ ] Task: Implement the Hybrid Confirmation logic in `App::update`.
    - [ ] Sub-task: Write Tests (Ensure popups only appear for destructive actions).
    - [ ] Sub-task: Implement Feature (Toggle between automatic and popup-required actions).
- [ ] Task: Implement `update` and `full-upgrade` maintenance commands.
    - [ ] Sub-task: Write Tests (Verify command strings).
    - [ ] Sub-task: Implement Feature (Add system maintenance to backend).
- [ ] Task: Conductor - User Manual Verification 'Phase 3: System Altering Operations & Confirmation' (Protocol in workflow.md)

## Phase 4: UX Polish & Progress Indicators
Focus: Visualizing background tasks and cleaning up the interface.

- [ ] Task: Add a persistent "Background Task" status zone in `ui.rs`.
    - [ ] Sub-task: Write Tests (Verify zone renders when a task is active).
    - [ ] Sub-task: Implement Feature (Add a status indicator at the bottom/top of the TUI).
- [ ] Task: Final cleanup and error reporting refinement.
    - [ ] Sub-task: Write Tests (Ensure failed APT commands trigger the notification queue).
    - [ ] Sub-task: Implement Feature (Map command exit codes to UI notifications).
- [ ] Task: Conductor - User Manual Verification 'Phase 4: UX Polish & Progress Indicators' (Protocol in workflow.md)
