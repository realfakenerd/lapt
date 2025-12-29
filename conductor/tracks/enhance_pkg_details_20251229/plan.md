# Plan: Enhance Package Details and Error Resilience

## Phase 1: Backend & Data Structure Updates [checkpoint: 516512a]
Focus: Extending the data model to support richer package metadata.

- [x] Task: Update `Package` struct in `pkg.rs` to include `summary`, `description`, `license`, `size`, `url`. [3d484a3]
    - [ ] Sub-task: Write Tests (Ensure `Package` struct can hold new fields and defaults are correct).
    - [ ] Sub-task: Implement Feature (Add fields to struct).
- [x] Task: Update `PackageKitBackend` in `backend.rs` to populate new fields. [8720f61]
    - [ ] Sub-task: Write Tests (Mock PackageKit response or unit test the mapping logic).
    - [ ] Sub-task: Implement Feature (Map PackageKit properties to the updated `Package` struct).
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Backend & Data Structure Updates' (Protocol in workflow.md)

## Phase 2: UI - Detailed Panel Implementation [checkpoint: e4bd9e2]
Focus: Visualizing the new metadata in the application's side panel.

- [x] Task: Create `ui::components::details.rs` (or similar) refactoring the details rendering logic. [fbaeb19]
    - [ ] Sub-task: Write Tests (Verify component renders text correctly based on input `Package`).
    - [ ] Sub-task: Implement Feature (Design the layout using Ratatui widgets).
- [x] Task: Integrate new details component into `ui.rs`. [d398e62]
- [x] Task: Implement `apt show` parsing logic to fetch missing details (License, Size, URL). [74f4cf2]
- [ ] Task: Conductor - User Manual Verification 'Phase 2: UI - Detailed Panel Implementation' (Protocol in workflow.md)

## Phase 3: Error Notification System
Focus: Implementing a robust, user-friendly error handling mechanism.

- [x] Task: Define `Notification` struct and add `notification_queue` to `App` state in `app.rs`. [0bd8a10]
    - [ ] Sub-task: Write Tests (Test pushing/popping notifications from the queue).
    - [ ] Sub-task: Implement Feature (Add struct and state field).
- [x] Task: Create `ErrorPopup` component in `ui.rs`. [9c983b3]
    - [ ] Sub-task: Write Tests (Verify popup dimensions and text rendering).
    - [ ] Sub-task: Implement Feature (Implement the rendering logic for the popup).
- [x] Task: Update `App::handle_backend_event` to route `BackendEvent::Error` to the notification queue. [1404293]
    - [ ] Sub-task: Write Tests (Simulate an error event and assert it ends up in the queue).
    - [ ] Sub-task: Implement Feature (Connect backend errors to the UI state).
- [x] Task: Add `Action::DismissNotification` and handle it in `App::update`. [a9eb5b5]
    - [ ] Sub-task: Write Tests (Test that the action removes the notification from the queue).
    - [ ] Sub-task: Implement Feature (Bind key event `Esc` or `Enter` to dismiss).
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Error Notification System' (Protocol in workflow.md)
