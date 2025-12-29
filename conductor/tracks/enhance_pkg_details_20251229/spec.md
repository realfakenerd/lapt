# Specification: Enhance Package Details and Error Resilience

## 1. Overview

This track focuses on two key improvements for the `lapt` application: enriching the package details panel with comprehensive metadata and establishing a robust, user-friendly error notification system. These enhancements align with the product goals of providing an "information-dense" UI and "supportive guidance" for errors.

## 2. Goals

- **Enrich Details Panel:** Display extended package information (e.g., version, architecture, summary, description, license, size, url) retrieved from the PackageKit backend.
- **Robust Error Handling:** Implement a centralized system for capturing and displaying errors within the TUI, ensuring technical details are available without overwhelming the user.
- **UX Consistency:** Ensure these new elements respect the "Zonal Consistency" and "Context-Aware Visibility" guidelines.

## 3. User Stories

- **As a user**, I want to see detailed information about a selected package (like its description, version, and size) so that I can make informed decisions before installing or removing it.
- **As a user**, I want to be clearly notified when an operation fails, with a helpful message and technical details available if I need to troubleshoot.
- **As a user**, I want error messages to be dismissible so they don't block my workflow permanently.

## 4. Technical Requirements

### 4.1. Package Details Enhancement

- **Backend:** Extend the `Package` struct (and `BackendEvent::InstalledPackagesFound` / `UpgradablePackagesFound`) or introduce a new `BackendCommand::GetPackageDetails` to fetch full metadata. Given PackageKit's nature, fetching full details for _all_ packages at once might be slow, so fetching on demand (when a package is selected or focused) or ensuring the initial list contains enough data is crucial.
    - _Decision:_ Let's verify if `packagekit-zbus` provides these details in the initial list or requires a separate call. We will assume for this spec that we might need to extend the `Package` struct and populate it.
- **Frontend (UI):** Update the `ui::draw_details` (or equivalent) function to render the new fields.
    - Fields to display: Name, Version, Arch, Summary, Description, License, Size (if available), URL.
    - Layout: Use a structured block (e.g., `Paragraph` or `Table`) within the Details panel.

### 4.2. Error Notification System

- **State Management:** Add an `error_queue` or `notification_stack` to the `App` state to manage multiple errors/messages.
- **UI Component:** Create a reusable `ErrorPopup` or a generic `Notification` component that overlays the interface or sits in a status zone.
- **Interaction:** Allow users to dismiss the error popup (e.g., via `Esc` or `Enter`).
- **Integration:** Update the `handle_backend_event` logic to push `BackendEvent::Error` payloads into this new system instead of just setting a single string.

## 5. Acceptance Criteria

- [ ] Selecting a package in the list updates the Details panel with Name, Version, Architecture, Summary, and Description (where available).
- [ ] Backend errors trigger a visible notification in the UI.
- [ ] The error notification displays a concise user-friendly message.
- [ ] The error notification can be dismissed by the user.
- [ ] The application remains stable and responsive even when multiple errors occur.
