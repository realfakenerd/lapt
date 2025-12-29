# Plan: Online Search and Real-time Task Feedback

## Phase 1: Tab Expansion & State Persistence [checkpoint: 8e7a9d1]
Focus: Adding the "Online" tab infrastructure and ensuring search results are stored correctly.

- [x] Task: Update `SelectedTab` and `App` state to support the "Online" tab. [f8c986e]
    - [ ] Sub-task: Write Tests (Ensure tab cycling includes the new variant and state initializes correctly).
    - [ ] Sub-task: Implement Feature (Add `Online` to enum and `online_packages` to `App` struct).
- [x] Task: Update UI to render the new tab and its list content. [ac8df93]
    - [ ] Sub-task: Write Tests (Verify tab bar and list area reflect the "Online" state).
    - [ ] Sub-task: Implement Feature (Modify `render_header` and `render_content` in `ui.rs`).
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Tab Expansion & State Persistence' (Protocol in workflow.md)

## Phase 2: Manual Online Search Integration [checkpoint: e0e810f]
Focus: Implementing the explicit search trigger and result handling.

- [~] Task: Modify search interaction to support manual triggers.
    - [ ] Sub-task: Write Tests (Ensure `Enter` in search mode dispatches a command ONLY when on the Online tab).
    - [ ] Sub-task: Implement Feature (Update `map_key_to_action` and `App::update`).
- [x] Task: Connect backend search results to the persistent `online_packages` state. [64ecb86]
    - [ ] Sub-task: Write Tests (Simulate search result event and verify state persistence).
    - [ ] Sub-task: Implement Feature (Update `handle_backend_event` in `app.rs`).
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Manual Online Search Integration' (Protocol in workflow.md)

## Phase 3: Real-time Signal System (Async Streaming) [checkpoint: 63d2100]
Focus: Refactoring the backend to provide live activity feedback.

- [x] Task: Implement async stdout streaming in `AptBackend`. [78fa9ef]
- [x] Task: Create a signal parser for APT output lines. [78fa9ef]
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Real-time Signal System (Async Streaming)' (Protocol in workflow.md)

## Phase 4: Refresh Action & Integration
Focus: Adding the refresh capability and final parity checks.

- [x] Task: Add repository refresh (`apt update`) action. [caae32c]
    - [ ] Sub-task: Write Tests (Ensure the refresh key dispatches the correct backend command).
    - [ ] Sub-task: Implement Feature (Map `f` key to `Action::RefreshRepos` and implement in backend).
- [x] Task: Ensure all management actions work across all tabs. [caae32c]
    - [ ] Sub-task: Write Tests (Verify 'Install' can be triggered from the Online tab).
    - [ ] Sub-task: Implement Feature (Final wiring and cleanup).
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Refresh Action & Integration' (Protocol in workflow.md)
