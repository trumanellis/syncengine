# Navigation Header Redesign - Implementation Plan

> **Created**: 2026-01-19
> **Status**: Ready for Implementation
> **Complexity**: Medium (UI refactor with file deletions)

---

## 1. Context

### 1.1 Original Request

Redesign the navigation header for Synchronicity Engine with:
- Three pages only: Tasks, Network, Profile (DELETE Packets page)
- Desktop: Horizontal header with app title, nav links with Lucide icons, status orb with peer dropdown
- Mobile (768px): Bottom navigation bar replacing header entirely
- Status orb that pulses only when actively syncing
- Peer status dropdown showing connected peers with message capability

### 1.2 Current State

**Files to modify:**
- `src/components/nav_header.rs` - Current navigation header component
- `src/components/mod.rs` - Component exports
- `src/pages/mod.rs` - Page exports
- `src/app.rs` - Route definitions
- `src/pages/network.rs` - Uses packets components for messaging
- `src/theme/styles.rs` - CSS styles

**Files to delete:**
- `src/pages/packets.rs` - Packets page (being removed)

**Files to rename:**
- `src/components/packets/` -> `src/components/messages/` (keep messaging components)

**New files to create:**
- `src/components/peer_status_dropdown.rs` - Peer list dropdown
- `src/components/mobile_nav.rs` - Mobile bottom navigation

### 1.3 Design System V2 Reference

```css
/* Colors */
--void-black: #0a0a0a;      /* Background */
--gold: #d4af37;            /* App title, important */
--cyan: #00d4aa;            /* Active nav, hover states */
--moss-glow: #7cb87c;       /* Status orb */
--text-primary: #f5f5f5;
--text-secondary: rgba(245, 245, 245, 0.7);

/* Fonts */
--font-serif: 'Cormorant Garamond', Georgia, serif;
--font-mono: 'JetBrains Mono', 'SF Mono', 'Consolas', monospace;

/* Transitions */
--transition-fast: 150ms ease;
--transition-normal: 200ms ease;
```

---

## 2. Work Objectives

### 2.1 Core Objective

Create a streamlined navigation system with desktop header and mobile bottom nav, removing the Packets page while preserving messaging functionality in Network.

### 2.2 Deliverables

1. Rewritten `NavHeader` component with Lucide icons
2. New `PeerStatusDropdown` component
3. New `MobileNav` component (bottom navigation)
4. Updated CSS styles
5. Cleaned up routes and page exports
6. Renamed packets components to messages

### 2.3 Definition of Done

- [ ] Desktop header displays: "Synchronicity Engine" (gold serif) | [Tasks] [Network] [Profile] | Status Orb with peer count
- [ ] Nav links use Lucide SVG icons with text labels
- [ ] Active nav link highlighted in cyan
- [ ] Status orb is moss green, pulses ONLY when `syncing` state is true
- [ ] Clicking orb opens dropdown with peer list
- [ ] Dropdown shows: "Connected (N)" header, peer rows with avatar/name/status/last-seen/[Message] button, [Sync Now] footer
- [ ] Mobile view (< 768px) hides header, shows bottom nav bar
- [ ] Bottom nav has 4 items: Tasks, Network, Profile, Status Orb
- [ ] /packets route removed, Packets page deleted
- [ ] Network page continues to work with messaging (uses renamed components)
- [ ] All existing functionality preserved

---

## 3. Guardrails

### 3.1 Must Have

- Lucide icons (SVG paths embedded, no external dependencies)
- 768px breakpoint for mobile
- Functional pulse animation (only when syncing)
- Accessible: 44x44px touch targets, focus states, ARIA labels
- Respect `prefers-reduced-motion`

### 3.2 Must NOT Have

- Decorative/continuous animations
- External icon library dependencies
- Breaking changes to engine API
- Changes to core business logic

---

## 4. Task Flow

```
[1. Preparation]
      |
      v
[2. Rename packets -> messages]
      |
      v
[3. Delete Packets page & route]
      |
      v
[4. Create PeerStatusDropdown]
      |
      v
[5. Create MobileNav]
      |
      v
[6. Rewrite NavHeader]
      |
      v
[7. Update CSS]
      |
      v
[8. Integration & Testing]
```

---

## 5. Detailed TODOs

### TODO 1: Preparation - Update module exports

**File: `src/components/mod.rs`**

Remove `packets` module, add `messages` module:

```rust
//! UI Components for Synchronicity Engine.
//!
//! Cyber-mystical terminal aesthetic components.

pub mod cards;
pub mod contacts;
mod field_status;
pub mod images;
mod intention_creator;
mod invite_panel;
mod markdown_editor;
mod mobile_nav;          // NEW
mod nav_header;
pub mod messages;        // RENAMED from packets
mod peer_status_dropdown; // NEW
pub mod profile;
mod realm_selector;
mod task_list;
mod unified_field;

pub use field_status::{
    FieldState, FieldStatus, NetworkResonance, NetworkResonanceCompact, NetworkState,
};
pub use intention_creator::{IntentionCategory, IntentionCreator, IntentionData};
pub use invite_panel::{InvitePanel, JoinRealmModal, QrCodeDisplay};
pub use markdown_editor::MarkdownEditor;
pub use mobile_nav::MobileNav;           // NEW
pub use nav_header::{NavHeader, NavLocation};
pub use peer_status_dropdown::PeerStatusDropdown; // NEW
pub use realm_selector::RealmSelector;
pub use task_list::{ManifestInput, TaskItem, TaskList};
pub use unified_field::UnifiedFieldView;
```

**Acceptance Criteria:**
- [ ] `packets` module renamed to `messages`
- [ ] New modules added for `mobile_nav` and `peer_status_dropdown`
- [ ] Exports updated

---

### TODO 2: Rename packets directory to messages

**Action:** Rename `src/components/packets/` to `src/components/messages/`

**File: `src/components/messages/mod.rs`** (after rename)

```rust
//! Messaging components (renamed from packets)

mod message_compose;
mod messages_list;

pub use message_compose::MessageCompose;
pub use messages_list::{MessagesList, ReceivedMessage};

// Note: KeysPanel, LogView, MirrorsGallery, PacketDetailModal are removed
// as they were specific to the Packets page
```

**Acceptance Criteria:**
- [ ] Directory renamed
- [ ] mod.rs updated to only export messaging components
- [ ] Unused components (KeysPanel, LogView, MirrorsGallery, PacketDetailModal) deleted

---

### TODO 3: Delete Packets page and update routes

**File: `src/pages/mod.rs`**

```rust
//! Page components for Synchronicity Engine.

mod field;
mod landing;
mod network;
mod profile;
mod realm_view;

pub use field::Field;
pub use landing::Landing;
pub use network::Network;
pub use profile::Profile;
pub use realm_view::RealmView;
```

**File: `src/app.rs`**

```rust
use std::sync::Arc;

use dioxus::prelude::*;
use tokio::sync::RwLock;

use crate::context::{get_data_dir, SharedEngine};
use crate::pages::{Field, Landing, Network, Profile, RealmView};
use crate::theme::GLOBAL_STYLES;

/// Application routes.
///
/// - `/` - Landing page with "Enter the Field" button
/// - `/field` - Main app view with realm sidebar and task list (Tasks)
/// - `/realms/:id` - Direct link to a specific realm
/// - `/profile` - Profile page with identity, peers, and stats
/// - `/network` - Network page with peers and messaging
#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[route("/")]
    Landing {},
    #[route("/field")]
    Field {},
    #[route("/realms/:id")]
    RealmView { id: String },
    #[route("/profile")]
    Profile {},
    #[route("/network")]
    Network {},
    // REMOVED: #[route("/packets")] Packets {},
}

/// Root application component.
///
/// Provides global styles, engine context, and routing.
#[component]
pub fn App() -> Element {
    // ... rest unchanged ...
}
```

**File to DELETE:** `src/pages/packets.rs`

**Acceptance Criteria:**
- [ ] `packets.rs` deleted
- [ ] `mod.rs` no longer exports Packets
- [ ] `app.rs` route removed
- [ ] Application compiles without Packets references

---

### TODO 4: Update Network page imports

**File: `src/pages/network.rs`**

Update the import from `packets` to `messages`:

```rust
// Change this:
use crate::components::packets::{MessageCompose, MessagesList, ReceivedMessage};

// To this:
use crate::components::messages::{MessageCompose, MessagesList, ReceivedMessage};
```

**Acceptance Criteria:**
- [ ] Import path updated
- [ ] Network page compiles and functions

---

### TODO 5: Create PeerStatusDropdown component

**File: `src/components/peer_status_dropdown.rs`**

```rust
//! Peer Status Dropdown Component
//!
//! Shows connected peers with status and message capability.

use dioxus::prelude::*;
use syncengine_core::{Peer, PeerStatus};

/// Format timestamp as relative time string
fn format_relative_time(timestamp: u64) -> String {
    let now = chrono::Utc::now().timestamp() as u64;
    let elapsed = now.saturating_sub(timestamp);

    if elapsed < 60 {
        "Just now".to_string()
    } else if elapsed < 3600 {
        format!("{}m ago", elapsed / 60)
    } else if elapsed < 86400 {
        format!("{}h ago", elapsed / 3600)
    } else {
        format!("{}d ago", elapsed / 86400)
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct PeerStatusDropdownProps {
    /// List of connected peers
    pub peers: Vec<Peer>,
    /// Whether currently syncing
    pub syncing: bool,
    /// Callback when dropdown is closed
    pub on_close: EventHandler<()>,
    /// Callback when Sync Now is clicked
    pub on_sync: EventHandler<()>,
    /// Callback when Message button is clicked for a peer
    pub on_message: EventHandler<(String, String)>, // (did, name)
}

/// Peer Status Dropdown
///
/// Displays list of connected peers with:
/// - Avatar (or initial)
/// - Display name
/// - Status dot (online/offline)
/// - Last seen time
/// - Message button
#[component]
pub fn PeerStatusDropdown(props: PeerStatusDropdownProps) -> Element {
    let online_count = props.peers.iter()
        .filter(|p| matches!(p.status, PeerStatus::Online))
        .count();

    rsx! {
        // Backdrop to close dropdown when clicking outside
        div {
            class: "peer-dropdown-backdrop",
            onclick: move |_| props.on_close.call(()),
        }

        div { class: "peer-dropdown",
            // Header
            header { class: "peer-dropdown-header",
                h3 { class: "peer-dropdown-title",
                    "Connected"
                    span { class: "peer-count-badge", "({online_count})" }
                }
            }

            // Peer list
            div { class: "peer-dropdown-list",
                if props.peers.is_empty() {
                    div { class: "peer-dropdown-empty",
                        p { "No peers connected" }
                        p { class: "peer-dropdown-hint", "Share your invite code to connect with others" }
                    }
                } else {
                    for peer in &props.peers {
                        {
                            let peer_did = peer.did.clone().unwrap_or_default();
                            let peer_name = peer.display_name();
                            let peer_did_for_click = peer_did.clone();
                            let peer_name_for_click = peer_name.clone();
                            let is_online = matches!(peer.status, PeerStatus::Online);
                            let first_char = peer_name.chars().next().unwrap_or('?').to_uppercase().to_string();
                            let last_seen = format_relative_time(peer.last_seen);

                            rsx! {
                                div {
                                    key: "{peer_did}",
                                    class: "peer-dropdown-item",

                                    // Avatar placeholder
                                    div { class: "peer-avatar-small",
                                        span { "{first_char}" }
                                    }

                                    // Info
                                    div { class: "peer-info",
                                        span { class: "peer-name", "{peer_name}" }
                                        div { class: "peer-meta",
                                            span {
                                                class: if is_online { "status-dot online" } else { "status-dot" }
                                            }
                                            span { class: "peer-last-seen", "{last_seen}" }
                                        }
                                    }

                                    // Message button
                                    button {
                                        class: "peer-message-btn",
                                        title: "Send message",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            props.on_message.call((
                                                peer_did_for_click.clone(),
                                                peer_name_for_click.clone(),
                                            ));
                                        },
                                        // Lucide message-square icon
                                        svg {
                                            xmlns: "http://www.w3.org/2000/svg",
                                            width: "16",
                                            height: "16",
                                            view_box: "0 0 24 24",
                                            fill: "none",
                                            stroke: "currentColor",
                                            stroke_width: "2",
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            path { d: "M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Footer with Sync Now button
            footer { class: "peer-dropdown-footer",
                button {
                    class: if props.syncing { "btn btn-primary sync-btn syncing" } else { "btn btn-primary sync-btn" },
                    disabled: props.syncing,
                    onclick: move |_| props.on_sync.call(()),
                    if props.syncing {
                        span { class: "sync-spinner" }
                        "Syncing..."
                    } else {
                        // Lucide refresh-cw icon
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "14",
                            height: "14",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            path { d: "M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" }
                            path { d: "M21 3v5h-5" }
                            path { d: "M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" }
                            path { d: "M8 16H3v5" }
                        }
                        "Sync Now"
                    }
                }
            }
        }
    }
}
```

**Acceptance Criteria:**
- [ ] Component renders peer list
- [ ] Shows avatar initial, name, status dot, last seen
- [ ] Message button triggers callback
- [ ] Sync Now button triggers callback
- [ ] Click outside closes dropdown

---

### TODO 6: Create MobileNav component

**File: `src/components/mobile_nav.rs`**

```rust
//! Mobile Navigation Component
//!
//! Bottom navigation bar for mobile devices (< 768px).

use dioxus::prelude::*;

use crate::app::Route;
use crate::components::nav_header::NavLocation;

#[derive(Props, Clone, PartialEq)]
pub struct MobileNavProps {
    /// Current active location
    pub current: NavLocation,
    /// Number of connected peers
    pub peer_count: usize,
    /// Whether currently syncing
    pub syncing: bool,
    /// Callback when status orb is clicked
    pub on_status_click: EventHandler<()>,
}

/// Mobile bottom navigation bar
///
/// Replaces header on screens < 768px.
/// Shows: Tasks | Network | Profile | Status Orb
#[component]
pub fn MobileNav(props: MobileNavProps) -> Element {
    let locations = [
        NavLocation::Field,
        NavLocation::Network,
        NavLocation::Profile,
    ];

    rsx! {
        nav { class: "mobile-nav",
            // Navigation items
            for location in &locations {
                Link {
                    to: location.route(),
                    class: if *location == props.current { "mobile-nav-item active" } else { "mobile-nav-item" },

                    // Icon
                    span { class: "mobile-nav-icon",
                        {render_nav_icon(*location)}
                    }

                    // Label (hidden by default, shown on active)
                    span { class: "mobile-nav-label", "{location.display_name()}" }
                }
            }

            // Status orb
            button {
                class: if props.syncing { "mobile-nav-status syncing" } else { "mobile-nav-status" },
                onclick: move |_| props.on_status_click.call(()),
                "aria-label": "Connection status",

                span { class: "status-orb" }
                if props.peer_count > 0 {
                    span { class: "peer-count-mini", "{props.peer_count}" }
                }
            }
        }
    }
}

/// Render Lucide icon for navigation location
fn render_nav_icon(location: NavLocation) -> Element {
    match location {
        NavLocation::Field => rsx! {
            // Lucide check-square icon (Tasks)
            svg {
                xmlns: "http://www.w3.org/2000/svg",
                width: "24",
                height: "24",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "m9 11 3 3L22 4" }
                path { d: "M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" }
            }
        },
        NavLocation::Network => rsx! {
            // Lucide users icon
            svg {
                xmlns: "http://www.w3.org/2000/svg",
                width: "24",
                height: "24",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" }
                circle { cx: "9", cy: "7", r: "4" }
                path { d: "M22 21v-2a4 4 0 0 0-3-3.87" }
                path { d: "M16 3.13a4 4 0 0 1 0 7.75" }
            }
        },
        NavLocation::Profile => rsx! {
            // Lucide user icon
            svg {
                xmlns: "http://www.w3.org/2000/svg",
                width: "24",
                height: "24",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                circle { cx: "12", cy: "8", r: "5" }
                path { d: "M20 21a8 8 0 0 0-16 0" }
            }
        },
    }
}
```

**Acceptance Criteria:**
- [ ] Bottom nav bar renders on mobile
- [ ] Shows Tasks, Network, Profile icons
- [ ] Active state highlighted
- [ ] Status orb shows peer count
- [ ] Status orb click triggers callback

---

### TODO 7: Rewrite NavHeader component

**File: `src/components/nav_header.rs`**

```rust
//! Navigation Header Component
//!
//! Desktop: Horizontal header with app title, nav links, status orb
//! Mobile: Hidden (replaced by MobileNav)

use dioxus::prelude::*;
use syncengine_core::{Peer, PeerStatus};

use crate::app::Route;
use crate::components::messages::{MessageCompose, ReceivedMessage};
use crate::components::PeerStatusDropdown;
use crate::context::{use_engine, use_engine_ready};

/// Navigation location within the application
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NavLocation {
    Field,
    Network,
    Profile,
}

impl NavLocation {
    /// Get the display name for this location
    pub fn display_name(&self) -> &'static str {
        match self {
            NavLocation::Field => "Tasks",
            NavLocation::Network => "Network",
            NavLocation::Profile => "Profile",
        }
    }

    /// Get the route for this location
    pub fn route(&self) -> Route {
        match self {
            NavLocation::Field => Route::Field {},
            NavLocation::Network => Route::Network {},
            NavLocation::Profile => Route::Profile {},
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct NavHeaderProps {
    /// Current location in the app
    pub current: NavLocation,
}

/// Navigation Header component
///
/// Desktop header with:
/// - Left: "Synchronicity Engine" title (gold serif)
/// - Center: Navigation links with Lucide icons
/// - Right: Status orb with peer count dropdown
#[component]
pub fn NavHeader(props: NavHeaderProps) -> Element {
    let engine = use_engine();
    let engine_ready = use_engine_ready();

    // State
    let mut show_dropdown = use_signal(|| false);
    let mut peers: Signal<Vec<Peer>> = use_signal(Vec::new);
    let mut syncing = use_signal(|| false);
    let mut compose_target: Signal<Option<(String, String)>> = use_signal(|| None);

    let locations = [
        NavLocation::Field,
        NavLocation::Network,
        NavLocation::Profile,
    ];

    // Load peers when engine ready
    use_effect(move || {
        if engine_ready() {
            spawn(async move {
                let shared = engine();
                let guard = shared.read().await;
                if let Some(ref eng) = *guard {
                    if let Ok(peer_list) = eng.list_peer_contacts() {
                        peers.set(peer_list);
                    }
                }
            });
        }
    });

    // Poll for peer updates
    use_effect(move || {
        spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                if engine_ready() {
                    let shared = engine();
                    let guard = shared.read().await;
                    if let Some(ref eng) = *guard {
                        if let Ok(peer_list) = eng.list_peer_contacts() {
                            peers.set(peer_list);
                        }
                    }
                }
            }
        });
    });

    // Sync handler
    let on_sync = move |_: ()| {
        if syncing() { return; }
        syncing.set(true);

        spawn(async move {
            let shared = engine();
            let mut guard = shared.write().await;
            if let Some(ref mut eng) = *guard {
                match eng.manual_sync().await {
                    Ok(_) => {
                        if let Ok(peer_list) = eng.list_peer_contacts() {
                            peers.set(peer_list);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Manual sync failed: {:?}", e);
                    }
                }
            }
            syncing.set(false);
        });
    };

    // Message handler
    let on_message = move |(did, name): (String, String)| {
        compose_target.set(Some((did, name)));
        show_dropdown.set(false);
    };

    // Send message handler
    let send_message = move |content: String| {
        if let Some((did, _name)) = compose_target() {
            spawn(async move {
                let shared = engine();
                let mut guard = shared.write().await;
                if let Some(ref mut eng) = *guard {
                    let payload = syncengine_core::PacketPayload::DirectMessage { content };
                    let address = syncengine_core::PacketAddress::Global;
                    match eng.create_and_broadcast_packet(payload, address).await {
                        Ok(seq) => {
                            tracing::info!(to = %did, sequence = seq, "Sent message");
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to send message");
                        }
                    }
                }
                compose_target.set(None);
            });
        }
    };

    let peer_count = peers().len();
    let online_count = peers().iter()
        .filter(|p| matches!(p.status, PeerStatus::Online))
        .count();

    rsx! {
        header { class: "nav-header-v2",
            div { class: "nav-header-inner",
                // Left: App title
                div { class: "nav-title",
                    h1 { class: "app-title", "Synchronicity Engine" }
                }

                // Center: Navigation links
                nav { class: "nav-links-v2",
                    for location in &locations {
                        Link {
                            to: location.route(),
                            class: if *location == props.current { "nav-link-v2 active" } else { "nav-link-v2" },

                            // Icon
                            span { class: "nav-link-icon",
                                {render_nav_icon(*location)}
                            }

                            // Label
                            span { class: "nav-link-label", "{location.display_name()}" }
                        }
                    }
                }

                // Right: Status orb with peer count
                div { class: "nav-status-v2",
                    button {
                        class: if syncing() { "status-orb-btn syncing" } else { "status-orb-btn" },
                        onclick: move |_| show_dropdown.toggle(),
                        "aria-label": "Connection status - {online_count} peers online",
                        "aria-expanded": "{show_dropdown()}",

                        span { class: if syncing() { "status-orb syncing" } else { "status-orb" } }
                        span { class: "peer-count", "{peer_count}" }

                        // Dropdown chevron
                        svg {
                            class: "dropdown-chevron",
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "12",
                            height: "12",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            path { d: "m6 9 6 6 6-6" }
                        }
                    }

                    // Dropdown
                    if show_dropdown() {
                        PeerStatusDropdown {
                            peers: peers(),
                            syncing: syncing(),
                            on_close: move |_| show_dropdown.set(false),
                            on_sync: on_sync,
                            on_message: on_message,
                        }
                    }
                }
            }
        }

        // Message compose modal
        if let Some((did, name)) = compose_target() {
            MessageCompose {
                recipient_name: name,
                recipient_did: did,
                on_send: send_message,
                on_close: move |_| compose_target.set(None),
            }
        }
    }
}

/// Render Lucide icon for navigation location
fn render_nav_icon(location: NavLocation) -> Element {
    match location {
        NavLocation::Field => rsx! {
            // Lucide check-square icon (Tasks)
            svg {
                xmlns: "http://www.w3.org/2000/svg",
                width: "18",
                height: "18",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "m9 11 3 3L22 4" }
                path { d: "M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" }
            }
        },
        NavLocation::Network => rsx! {
            // Lucide users icon
            svg {
                xmlns: "http://www.w3.org/2000/svg",
                width: "18",
                height: "18",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" }
                circle { cx: "9", cy: "7", r: "4" }
                path { d: "M22 21v-2a4 4 0 0 0-3-3.87" }
                path { d: "M16 3.13a4 4 0 0 1 0 7.75" }
            }
        },
        NavLocation::Profile => rsx! {
            // Lucide user icon
            svg {
                xmlns: "http://www.w3.org/2000/svg",
                width: "18",
                height: "18",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                circle { cx: "12", cy: "8", r: "5" }
                path { d: "M20 21a8 8 0 0 0-16 0" }
            }
        },
    }
}
```

**Acceptance Criteria:**
- [ ] Desktop header renders with title, nav links, status orb
- [ ] Lucide icons display correctly
- [ ] Active nav link has cyan highlight
- [ ] Status orb shows peer count
- [ ] Clicking orb opens dropdown
- [ ] Syncing state triggers pulse animation
- [ ] Hidden on mobile (CSS handles this)

---

### TODO 8: Add CSS Styles

**Additions to `src/theme/styles.rs`**

Add the following CSS block (can be added after existing nav-header styles):

```css
/* ═══════════════════════════════════════════════════════════════════════════
   NAV HEADER V2 - Redesigned navigation
   ═══════════════════════════════════════════════════════════════════════════ */

.nav-header-v2 {
  position: sticky;
  top: 0;
  z-index: 100;
  width: 100%;
  background: var(--void-black);
  border-bottom: 1px solid var(--void-border);
}

.nav-header-inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  max-width: 1400px;
  margin: 0 auto;
  padding: var(--space-3) var(--space-6);
  gap: var(--space-6);
}

/* App title - left */
.nav-title {
  flex-shrink: 0;
}

.app-title {
  font-family: var(--font-serif);
  font-size: var(--text-lg);
  font-weight: 400;
  color: var(--gold);
  letter-spacing: 0.05em;
  margin: 0;
  white-space: nowrap;
}

/* Navigation links - center */
.nav-links-v2 {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.nav-link-v2 {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  background: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  text-decoration: none;
  transition: color var(--transition-fast),
              border-color var(--transition-fast),
              background var(--transition-fast);
  cursor: pointer;
}

.nav-link-v2:hover {
  color: var(--cyan);
  border-color: rgba(0, 212, 170, 0.3);
  background: rgba(0, 212, 170, 0.05);
}

.nav-link-v2.active {
  color: var(--cyan);
  border-color: var(--cyan);
  background: rgba(0, 212, 170, 0.1);
}

.nav-link-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
}

.nav-link-icon svg {
  display: block;
}

.nav-link-label {
  font-weight: 400;
}

/* Status orb - right */
.nav-status-v2 {
  position: relative;
  flex-shrink: 0;
}

.status-orb-btn {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  background: rgba(124, 184, 124, 0.1);
  border: 1px solid var(--moss);
  border-radius: 20px;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: border-color var(--transition-fast),
              background var(--transition-fast);
}

.status-orb-btn:hover {
  border-color: var(--moss-glow);
  background: rgba(124, 184, 124, 0.15);
}

.status-orb {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--moss-glow);
  transition: box-shadow var(--transition-fast);
}

/* Pulse animation ONLY when syncing */
.status-orb.syncing {
  animation: orb-pulse 1.5s ease-in-out infinite;
}

@keyframes orb-pulse {
  0%, 100% {
    box-shadow: 0 0 4px var(--moss-glow);
    transform: scale(1);
  }
  50% {
    box-shadow: 0 0 12px var(--moss-glow), 0 0 20px rgba(124, 184, 124, 0.3);
    transform: scale(1.1);
  }
}

.peer-count {
  font-weight: 500;
  min-width: 1ch;
}

.dropdown-chevron {
  opacity: 0.6;
  transition: transform var(--transition-fast);
}

.status-orb-btn[aria-expanded="true"] .dropdown-chevron {
  transform: rotate(180deg);
}

/* ═══════════════════════════════════════════════════════════════════════════
   PEER STATUS DROPDOWN
   ═══════════════════════════════════════════════════════════════════════════ */

.peer-dropdown-backdrop {
  position: fixed;
  inset: 0;
  z-index: 99;
}

.peer-dropdown {
  position: absolute;
  top: calc(100% + var(--space-2));
  right: 0;
  z-index: 100;
  width: 320px;
  max-height: 400px;
  background: var(--void-black);
  border: 1px solid var(--moss);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  overflow: hidden;
  animation: fadeIn var(--transition-normal);
}

.peer-dropdown-header {
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--void-border);
}

.peer-dropdown-title {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  font-weight: 500;
  color: var(--text-primary);
  margin: 0;
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.peer-count-badge {
  color: var(--moss-glow);
}

.peer-dropdown-list {
  max-height: 280px;
  overflow-y: auto;
  padding: var(--space-2);
}

.peer-dropdown-empty {
  padding: var(--space-6) var(--space-4);
  text-align: center;
}

.peer-dropdown-empty p {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-secondary);
  margin: 0;
}

.peer-dropdown-hint {
  margin-top: var(--space-2) !important;
  color: var(--text-muted) !important;
  font-size: var(--text-sm) !important;
}

.peer-dropdown-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-2) var(--space-3);
  border-radius: 6px;
  transition: background var(--transition-fast);
}

.peer-dropdown-item:hover {
  background: rgba(255, 255, 255, 0.03);
}

.peer-avatar-small {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--void-lighter);
  border: 1px solid var(--void-border);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.peer-avatar-small span {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-secondary);
  text-transform: uppercase;
}

.peer-info {
  flex: 1;
  min-width: 0;
}

.peer-name {
  display: block;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.peer-meta {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-top: 2px;
}

.peer-meta .status-dot {
  width: 6px;
  height: 6px;
}

.peer-last-seen {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--text-muted);
}

.peer-message-btn {
  padding: var(--space-2);
  background: transparent;
  border: 1px solid var(--void-border);
  border-radius: 4px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: border-color var(--transition-fast),
              color var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
}

.peer-message-btn:hover {
  border-color: var(--cyan);
  color: var(--cyan);
}

.peer-dropdown-footer {
  padding: var(--space-3) var(--space-4);
  border-top: 1px solid var(--void-border);
}

.sync-btn {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
}

.sync-btn svg {
  flex-shrink: 0;
}

.sync-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--void-border);
  border-top-color: var(--moss-glow);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

/* ═══════════════════════════════════════════════════════════════════════════
   MOBILE NAVIGATION - Bottom bar
   ═══════════════════════════════════════════════════════════════════════════ */

.mobile-nav {
  display: none; /* Hidden on desktop */
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: 100;
  background: var(--void-black);
  border-top: 1px solid var(--void-border);
  padding: var(--space-2) var(--space-4);
  padding-bottom: calc(var(--space-2) + env(safe-area-inset-bottom, 0px));
}

.mobile-nav-item {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-1);
  padding: var(--space-2);
  background: transparent;
  border: none;
  color: var(--text-muted);
  text-decoration: none;
  transition: color var(--transition-fast);
  min-width: 44px;
  min-height: 44px;
}

.mobile-nav-item:hover,
.mobile-nav-item.active {
  color: var(--cyan);
}

.mobile-nav-icon {
  display: flex;
  align-items: center;
  justify-content: center;
}

.mobile-nav-label {
  font-family: var(--font-mono);
  font-size: 0.625rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.mobile-nav-status {
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-1);
  padding: var(--space-2);
  background: transparent;
  border: none;
  cursor: pointer;
  min-width: 44px;
  min-height: 44px;
}

.mobile-nav-status .status-orb {
  width: 12px;
  height: 12px;
}

.mobile-nav-status.syncing .status-orb {
  animation: orb-pulse 1.5s ease-in-out infinite;
}

.peer-count-mini {
  font-family: var(--font-mono);
  font-size: 0.625rem;
  color: var(--text-secondary);
}

/* ═══════════════════════════════════════════════════════════════════════════
   RESPONSIVE - Mobile breakpoint at 768px
   ═══════════════════════════════════════════════════════════════════════════ */

@media (max-width: 768px) {
  /* Hide desktop header */
  .nav-header-v2 {
    display: none;
  }

  /* Show mobile nav */
  .mobile-nav {
    display: flex;
    align-items: center;
    justify-content: space-around;
  }

  /* Add padding to page content to account for bottom nav */
  .page,
  .field-page,
  .network-page,
  .profile-page {
    padding-bottom: calc(80px + env(safe-area-inset-bottom, 0px));
  }
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {
  .status-orb.syncing,
  .mobile-nav-status.syncing .status-orb {
    animation: none;
  }

  .sync-spinner {
    animation: none;
  }
}
```

**Acceptance Criteria:**
- [ ] Desktop header styled correctly
- [ ] Nav links have proper hover/active states
- [ ] Status orb styled with moss green
- [ ] Pulse animation only on syncing
- [ ] Dropdown styled correctly
- [ ] Mobile nav hidden on desktop
- [ ] Mobile nav shows at 768px breakpoint
- [ ] Safe area insets for mobile devices
- [ ] Reduced motion respected

---

### TODO 9: Update page components to use new NavHeader

Each page component (`field.rs`, `network.rs`, `profile.rs`) needs to:
1. Remove `status`, `action_text`, `action_loading`, `on_action` props from NavHeader
2. The new NavHeader handles status/sync internally

**Example change for Network page:**

```rust
// Before:
NavHeader {
    current: NavLocation::Network,
    status: Some(status_text),
    action_text: Some(action_text.to_string()),
    action_loading: syncing(),
    on_action: on_sync_click,
}

// After:
NavHeader {
    current: NavLocation::Network,
}
```

**Acceptance Criteria:**
- [ ] Field page updated
- [ ] Network page updated (remove sync logic handled by header)
- [ ] Profile page updated
- [ ] All pages compile

---

### TODO 10: Integration with MobileNav in App.rs

**File: `src/app.rs`**

Add MobileNav as a global component that appears on all pages:

```rust
use crate::components::MobileNav;

// In the App component, wrap Router with a parent that includes MobileNav:
#[component]
pub fn App() -> Element {
    // ... existing engine initialization ...

    // Track current route for mobile nav
    let current_location = use_memo(move || {
        // This would need route context - simplified for now
        NavLocation::Field
    });

    rsx! {
        style { {GLOBAL_STYLES} }

        // Main content area
        Router::<Route> {}

        // Mobile navigation (always rendered, CSS hides on desktop)
        MobileNav {
            current: current_location(),
            peer_count: 0, // Would need state
            syncing: false,
            on_status_click: move |_| {
                // Could navigate to Network or show dropdown
            },
        }
    }
}
```

**Note:** The mobile nav needs access to peer state. Consider:
1. Moving peer state to a context provider
2. Or having each page render its own MobileNav with its state

**Alternative approach:** Each page renders MobileNav with its own state (simpler, avoids global state changes).

**Acceptance Criteria:**
- [ ] MobileNav appears on mobile devices
- [ ] Correct page is highlighted
- [ ] Status orb functional

---

## 6. Commit Strategy

| Commit | Changes |
|--------|---------|
| `refactor: rename packets components to messages` | Rename directory, update mod.rs, delete unused components |
| `feat: delete Packets page and route` | Remove packets.rs, update mod.rs, update app.rs |
| `feat: add PeerStatusDropdown component` | New component file |
| `feat: add MobileNav component` | New component file |
| `feat: rewrite NavHeader with Lucide icons` | Rewrite nav_header.rs |
| `style: add CSS for nav header v2` | Update styles.rs |
| `fix: update page components for new NavHeader` | Update field.rs, network.rs, profile.rs |
| `feat: integrate MobileNav in app` | Update app.rs |

---

## 7. Success Criteria

### Functional
- [ ] Application compiles without errors
- [ ] Desktop header displays correctly
- [ ] Nav links navigate to correct pages
- [ ] Status orb shows peer count
- [ ] Dropdown opens/closes correctly
- [ ] Peer list displays with message buttons
- [ ] Sync Now triggers sync
- [ ] Mobile nav appears at 768px breakpoint
- [ ] Mobile nav items work correctly

### Visual
- [ ] "Synchronicity Engine" in gold serif font
- [ ] Nav links with Lucide icons
- [ ] Cyan active/hover states
- [ ] Moss green status orb
- [ ] Pulse only when syncing
- [ ] Dropdown styled correctly
- [ ] Mobile nav matches design

### Accessibility
- [ ] Focus states visible
- [ ] ARIA labels present
- [ ] 44x44px touch targets
- [ ] Reduced motion respected

---

## 8. Testing Checklist

### Desktop Testing
- [ ] Header visible at > 768px
- [ ] Click Tasks link -> navigates to /field
- [ ] Click Network link -> navigates to /network
- [ ] Click Profile link -> navigates to /profile
- [ ] Active link highlighted
- [ ] Hover states work
- [ ] Click status orb -> dropdown opens
- [ ] Click outside dropdown -> closes
- [ ] Click Sync Now -> syncing state shown
- [ ] Click Message button -> compose modal opens

### Mobile Testing
- [ ] Resize to < 768px
- [ ] Header hidden
- [ ] Bottom nav visible
- [ ] Nav items functional
- [ ] Status orb shows count
- [ ] Safe area insets work (notched devices)

### Accessibility Testing
- [ ] Tab through nav links
- [ ] Focus rings visible
- [ ] Screen reader announces labels
- [ ] Enable reduced motion -> animations stop

---

## 9. Handoff

After completing this plan:
1. Run `/start-work` to begin implementation
2. Follow commit strategy for atomic commits
3. Test each change before moving to next

---

*Plan generated by Prometheus*
*Ready for implementation*
