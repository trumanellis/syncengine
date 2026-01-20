# Network Page Chat Redesign - Implementation Plan

## Context

### Original Request
Redesign the Network page from a two-column "contacts + message feed" layout to a side-by-side chat interface with a contacts sidebar and conversation panel.

### Decisions (Pre-Made)
| Decision | Choice |
|----------|--------|
| Layout | Side-by-side split (30% contacts, 70% conversation) |
| Mobile | Full-screen conversation at 768px breakpoint |
| Sorting | Contacts sorted by last message time (recent first) |
| Unread | Simple dot indicator |
| Empty state | "Select a contact to begin transmission" |
| Updates | Contact events + 5-second polling fallback |

### Current State Analysis
- `network.rs` uses a two-column grid with contacts list and `MessagesList` component
- `MessageCompose` is a modal that appears when clicking "Message" button
- `ConversationView` component exists with full chat UI (header, bubbles, input)
- All conversation CSS already exists in `styles.rs` (lines 6866-7119)
- Engine API uses `mirror_packets_since()` to fetch messages

---

## 1. ASCII Layout Diagrams

### Desktop Layout (>768px)
```
+------------------------------------------------------------------+
|  NavHeader                                                        |
+------------------------------------------------------------------+
|                    network-content                                |
+-------------------+----------------------------------------------+
|                   |                                              |
|  CONTACTS (30%)   |  CONVERSATION (70%)                          |
|                   |                                              |
| +--------------+  | +------------------------------------------+ |
| | ContactRow   |  | | conversation-header                      | |
| | [*] Alice    |  | | [<] Alice                                | |
| |     2m ago   |  | |     did:sync:abc123...                   | |
| |     "Hey!"   |  | +------------------------------------------+ |
| +--------------+  | |                                          | |
| | ContactRow   |  | |  conversation-messages                   | |
| | [ ] Bob      |  | |                                          | |
| |     1h ago   |  | |  +----------------+                      | |
| |     "Sure"   |  | |  | Their message  |                      | |
| +--------------+  | |  +----------------+                      | |
| | ContactRow   |  | |                                          | |
| | [ ] Carol    |  | |           +----------------+             | |
| |     3d ago   |  | |           | Your message   |             | |
| |     "Thanks" |  | |           +----------------+             | |
| +--------------+  | |                                          | |
|                   | +------------------------------------------+ |
|                   | | message-input-bar                        | |
|                   | | [Type a message...        ] [Send]       | |
|                   | +------------------------------------------+ |
+-------------------+----------------------------------------------+

Legend:
  [*] = unread indicator (dot)
  [ ] = no unread
  [<] = back button (mobile only visible)
```

### Desktop Empty State (No Contact Selected)
```
+------------------------------------------------------------------+
|  NavHeader                                                        |
+------------------------------------------------------------------+
|                    network-content                                |
+-------------------+----------------------------------------------+
|                   |                                              |
|  CONTACTS (30%)   |                                              |
|                   |        +------------------------+            |
| +--------------+  |        |                        |            |
| | ContactRow   |  |        |         ~              |            |
| | [ ] Alice    |  |        |                        |            |
| +--------------+  |        | Select a contact to    |            |
| | ContactRow   |  |        | begin transmission     |            |
| | [ ] Bob      |  |        |                        |            |
| +--------------+  |        +------------------------+            |
|                   |                                              |
+-------------------+----------------------------------------------+
```

### Mobile Layout (<768px) - Contacts View
```
+------------------------+
|  NavHeader             |
+------------------------+
|                        |
|  CONTACTS (100%)       |
|                        |
| +--------------------+ |
| | ContactRow         | |
| | [*] Alice    2m    | |
| |     "Hey there!" > | |
| +--------------------+ |
| | ContactRow         | |
| | [ ] Bob       1h   | |
| |     "Sure thing" > | |
| +--------------------+ |
|                        |
+------------------------+
```

### Mobile Layout (<768px) - Conversation View
```
+------------------------+
|  NavHeader             |
+------------------------+
| conversation-header    |
| [<] Alice              |
|     did:sync:abc...    |
+------------------------+
|                        |
|  conversation-messages |
|                        |
|  +----------------+    |
|  | Their message  |    |
|  +----------------+    |
|                        |
|     +----------------+ |
|     | Your message   | |
|     +----------------+ |
|                        |
+------------------------+
| [Type a message...   ] |
| [Send]                 |
+------------------------+
```

---

## 2. Component Hierarchy

```
NetworkPage
|
+-- NavHeader (existing)
|
+-- network-content
    |
    +-- contacts-sidebar (30% width)
    |   |
    |   +-- sidebar-header
    |   |   +-- "Contacts" title
    |   |   +-- contact count badge
    |   |
    |   +-- contacts-list
    |       +-- ContactRow (for each contact)
    |           +-- avatar (AsyncImage or placeholder)
    |           +-- contact-info
    |           |   +-- name + unread dot
    |           |   +-- last message preview
    |           |   +-- timestamp
    |           +-- chevron (mobile only)
    |
    +-- conversation-panel (70% width)
        |
        +-- [IF no contact selected]
        |   EmptyConversationState
        |       +-- icon (~)
        |       +-- "Select a contact to begin transmission"
        |
        +-- [IF contact selected]
            ConversationView (existing component)
                +-- conversation-header (with back button)
                +-- conversation-messages (scrollable)
                |   +-- MessageBubble (for each message)
                +-- conversation-input-container
                    +-- MessageInput
```

---

## 3. State Management

### Signals Required

```rust
// Selected contact state
let mut selected_contact: Signal<Option<SelectedContact>> = use_signal(|| None);

// Where SelectedContact is:
struct SelectedContact {
    did: String,
    name: String,
}

// Contacts list (sorted by last message time)
let mut contacts: Signal<Vec<ContactWithPreview>> = use_signal(Vec::new);

// Where ContactWithPreview includes:
struct ContactWithPreview {
    peer: Peer,                    // From existing Peer type
    last_message_time: Option<i64>,
    last_message_preview: Option<String>,
    unread_count: u32,
}

// Conversation state for selected contact
let mut conversation_messages: Signal<Vec<ChatBubbleMessage>> = use_signal(Vec::new);

// Loading states
let mut loading = use_signal(|| true);           // Initial contacts load
let mut conversation_loading = use_signal(|| false);  // Loading messages
let mut sending = use_signal(|| false);          // Sending a message
```

### State Flow

```
1. Page Load:
   - loading = true
   - Fetch contacts with last message info
   - Sort by last_message_time DESC
   - loading = false

2. Contact Selection:
   - selected_contact = Some(contact)
   - conversation_loading = true
   - Fetch messages for contact via mirror_packets_since
   - Convert to ChatBubbleMessage format
   - conversation_loading = false

3. Send Message:
   - sending = true
   - Call engine.create_and_broadcast_packet()
   - Optimistically add to conversation_messages
   - sending = false

4. Real-time Updates:
   - Subscribe to contact events
   - On ProfileUpdated: refresh contacts list
   - 5-second polling: refresh current conversation messages
```

---

## 4. File Changes (network.rs)

### 4.1 Import Changes

**Remove:**
```rust
use crate::components::messages::{MessageCompose, MessagesList, ReceivedMessage};
```

**Change to:**
```rust
use crate::components::messages::{ChatBubbleMessage, ConversationView};
```

### 4.2 Add New Struct for Contact Display

```rust
/// Contact with conversation preview info
#[derive(Clone)]
struct ContactWithPreview {
    peer: Peer,
    last_message_time: Option<i64>,
    last_message_preview: Option<String>,
    unread_count: u32,
}

/// Currently selected contact
#[derive(Clone)]
struct SelectedContact {
    did: String,
    name: String,
}
```

### 4.3 Replace State Signals

**Remove:**
```rust
let mut messages: Signal<Vec<ReceivedMessage>> = use_signal(Vec::new);
let mut compose_target: Signal<Option<(String, String)>> = use_signal(|| None);
```

**Add:**
```rust
let mut selected_contact: Signal<Option<SelectedContact>> = use_signal(|| None);
let mut conversation_messages: Signal<Vec<ChatBubbleMessage>> = use_signal(Vec::new);
let mut conversation_loading = use_signal(|| false);
let mut sending = use_signal(|| false);
```

### 4.4 Add Contact Loading with Preview

Replace the existing contact loading logic to also compute last message info:

```rust
// In the initial load effect
for contact in &contact_list {
    if let Some(ref did_str) = contact.did {
        if let Ok(did) = did_str.parse::<Did>() {
            if let Ok(packets) = eng.mirror_packets_since(&did, 0) {
                // Find the most recent DirectMessage
                let last_msg = packets
                    .iter()
                    .filter_map(|env| {
                        if env.is_global() {
                            if let Ok(payload) = env.decode_global_payload() {
                                if let PacketPayload::DirectMessage { content } = payload {
                                    return Some((env.timestamp, content));
                                }
                            }
                        }
                        None
                    })
                    .max_by_key(|(ts, _)| *ts);

                contacts_with_preview.push(ContactWithPreview {
                    peer: contact.clone(),
                    last_message_time: last_msg.as_ref().map(|(ts, _)| *ts),
                    last_message_preview: last_msg.map(|(_, content)| {
                        if content.len() > 50 {
                            format!("{}...", &content[..47])
                        } else {
                            content
                        }
                    }),
                    unread_count: 0, // TODO: implement unread tracking
                });
            }
        }
    }
}

// Sort by last message time (most recent first)
contacts_with_preview.sort_by(|a, b| {
    b.last_message_time.cmp(&a.last_message_time)
});
```

### 4.5 Add Conversation Loading Effect

```rust
// Load conversation when contact is selected
use_effect(move || {
    if let Some(ref contact) = selected_contact() {
        let contact_did = contact.did.clone();

        spawn(async move {
            conversation_loading.set(true);

            let shared = engine();
            let guard = shared.read().await;

            if let Some(ref eng) = *guard {
                if let Ok(did) = contact_did.parse::<Did>() {
                    if let Ok(packets) = eng.mirror_packets_since(&did, 0) {
                        let messages: Vec<ChatBubbleMessage> = packets
                            .iter()
                            .filter_map(|env| {
                                if env.is_global() {
                                    if let Ok(payload) = env.decode_global_payload() {
                                        if let PacketPayload::DirectMessage { content } = payload {
                                            return Some(ChatBubbleMessage {
                                                id: format!("{}-{}", contact_did, env.sequence),
                                                content,
                                                sender_name: None, // Will be set based on is_mine
                                                timestamp: env.timestamp,
                                                is_mine: false, // TODO: determine from packet signer
                                            });
                                        }
                                    }
                                }
                                None
                            })
                            .collect();

                        conversation_messages.set(messages);
                    }
                }
            }

            conversation_loading.set(false);
        });
    }
});
```

### 4.6 Update Send Message Handler

```rust
let send_message = move |content: String| {
    if let Some(ref contact) = selected_contact() {
        let contact_did = contact.did.clone();

        spawn(async move {
            sending.set(true);

            let shared = engine();
            let mut guard = shared.write().await;

            if let Some(ref mut eng) = *guard {
                let payload = PacketPayload::DirectMessage { content: content.clone() };

                if let Ok(parsed_did) = Did::parse(&contact_did) {
                    let address = PacketAddress::Individual(parsed_did);

                    match eng.create_and_broadcast_packet(payload, address).await {
                        Ok(seq) => {
                            // Optimistically add message to conversation
                            let new_msg = ChatBubbleMessage {
                                id: format!("sent-{}", seq),
                                content,
                                sender_name: None,
                                timestamp: chrono::Utc::now().timestamp_millis(),
                                is_mine: true,
                            };

                            let mut msgs = conversation_messages();
                            msgs.push(new_msg);
                            conversation_messages.set(msgs);

                            tracing::info!(to = %contact_did, sequence = seq, "Sent message");
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to send message");
                        }
                    }
                }
            }

            sending.set(false);
        });
    }
};
```

### 4.7 Replace RSX Layout

Replace the entire `network-grid` section with:

```rust
div { class: "network-chat-layout",
    // Contacts Sidebar (30%)
    aside {
        class: if selected_contact().is_some() {
            "contacts-sidebar mobile-hidden"
        } else {
            "contacts-sidebar"
        },

        header { class: "sidebar-header",
            h2 { class: "sidebar-title", "Contacts" }
            if contact_count > 0 {
                span { class: "contact-count-badge", "{contact_count}" }
            }
        }

        if contacts().is_empty() {
            div { class: "empty-state",
                p { class: "empty-state-message", "No contacts yet" }
                p { class: "empty-state-hint", "Add contacts to start messaging." }
            }
        } else {
            div { class: "contacts-list",
                for contact in contacts() {
                    {
                        let did = contact.peer.did.clone().unwrap_or_default();
                        let name = contact.peer.display_name();
                        let is_selected = selected_contact()
                            .as_ref()
                            .map(|s| s.did == did)
                            .unwrap_or(false);
                        let has_unread = contact.unread_count > 0;
                        let did_clone = did.clone();
                        let name_clone = name.clone();
                        let avatar_blob_id = contact.peer.profile
                            .as_ref()
                            .and_then(|p| p.avatar_blob_id.clone());
                        let first_char = name.chars().next()
                            .unwrap_or('?')
                            .to_uppercase()
                            .to_string();

                        rsx! {
                            div {
                                key: "{did}",
                                class: if is_selected {
                                    "contact-row contact-row-selected"
                                } else {
                                    "contact-row"
                                },
                                onclick: move |_| {
                                    selected_contact.set(Some(SelectedContact {
                                        did: did_clone.clone(),
                                        name: name_clone.clone(),
                                    }));
                                },

                                // Avatar
                                div { class: "contact-row-avatar",
                                    if let Some(ref blob_id) = avatar_blob_id {
                                        AsyncImage {
                                            blob_id: blob_id.clone(),
                                            alt: name.clone(),
                                            class: Some("avatar-image".to_string()),
                                        }
                                    } else {
                                        div { class: "avatar-placeholder",
                                            "{first_char}"
                                        }
                                    }
                                }

                                // Info
                                div { class: "contact-row-info",
                                    div { class: "contact-row-header",
                                        span { class: "contact-row-name", "{name}" }
                                        if has_unread {
                                            span { class: "unread-dot" }
                                        }
                                        if let Some(time) = contact.last_message_time {
                                            span { class: "contact-row-time",
                                                "{format_relative_time(time as u64)}"
                                            }
                                        }
                                    }
                                    if let Some(ref preview) = contact.last_message_preview {
                                        p { class: "contact-row-preview", "{preview}" }
                                    }
                                }

                                // Chevron (mobile indicator)
                                div { class: "contact-row-chevron",
                                    svg {
                                        width: "16",
                                        height: "16",
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        polyline { points: "9 18 15 12 9 6" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Conversation Panel (70%)
    main {
        class: if selected_contact().is_some() {
            "conversation-panel"
        } else {
            "conversation-panel mobile-hidden"
        },

        if let Some(ref contact) = selected_contact() {
            ConversationView {
                contact_did: contact.did.clone(),
                contact_name: contact.name.clone(),
                messages: conversation_messages(),
                on_send: send_message,
                on_back: move |_| selected_contact.set(None),
                sending: sending(),
                loading: conversation_loading(),
            }
        } else {
            // Empty state
            div { class: "empty-conversation",
                p { class: "empty-icon", "~" }
                p { class: "empty-text", "Select a contact to begin transmission" }
            }
        }
    }
}
```

### 4.8 Remove Modal

Delete the entire `MessageCompose` conditional at the end of the RSX:

```rust
// DELETE THIS BLOCK:
// Message compose modal
if let Some((did, name)) = compose_target() {
    MessageCompose { ... }
}
```

---

## 5. CSS Updates (styles.rs)

Add these new styles after the existing network styles (around line 5310):

```css
/* === Network Chat Layout === */
.network-chat-layout {
  display: grid;
  grid-template-columns: 300px 1fr;
  height: calc(100vh - 60px);  /* Subtract header height */
  max-width: 1400px;
  margin: 0 auto;
}

/* Contacts Sidebar */
.contacts-sidebar {
  display: flex;
  flex-direction: column;
  background: var(--void-lighter);
  border-right: 1px solid var(--void-border);
  overflow: hidden;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-4);
  border-bottom: 1px solid var(--void-border);
}

.sidebar-title {
  font-family: var(--font-serif);
  font-size: var(--text-lg);
  color: var(--gold);
  margin: 0;
}

.contact-count-badge {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-muted);
  background: var(--void-black);
  padding: 2px 8px;
  border-radius: 10px;
}

.contacts-list {
  flex: 1;
  overflow-y: auto;
}

/* Contact Row */
.contact-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  cursor: pointer;
  transition: background var(--transition-fast);
  border-bottom: 1px solid var(--void-border);
}

.contact-row:hover {
  background: rgba(0, 212, 170, 0.05);
}

.contact-row-selected {
  background: rgba(0, 212, 170, 0.1);
  border-left: 3px solid var(--cyan);
}

.contact-row-avatar {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  overflow: hidden;
  background: var(--void-black);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.contact-row-avatar .avatar-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.contact-row-avatar .avatar-placeholder {
  font-family: var(--font-serif);
  font-size: var(--text-lg);
  color: var(--gold);
}

.contact-row-info {
  flex: 1;
  min-width: 0;  /* Enable text truncation */
}

.contact-row-header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.contact-row-name {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.unread-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--cyan);
  flex-shrink: 0;
}

.contact-row-time {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--text-muted);
  margin-left: auto;
  flex-shrink: 0;
}

.contact-row-preview {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--text-muted);
  margin: var(--space-1) 0 0 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.contact-row-chevron {
  color: var(--text-muted);
  flex-shrink: 0;
  display: none;  /* Hidden on desktop */
}

/* Conversation Panel */
.conversation-panel {
  display: flex;
  flex-direction: column;
  background: var(--void-black);
  overflow: hidden;
}

/* Empty Conversation State */
.empty-conversation {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: var(--space-3);
}

.empty-conversation .empty-icon {
  font-size: 4rem;
  color: var(--text-muted);
  opacity: 0.5;
  margin: 0;
}

.empty-conversation .empty-text {
  font-family: var(--font-mono);
  font-size: var(--text-base);
  color: var(--text-secondary);
  margin: 0;
}

/* Mobile Responsive */
@media (max-width: 768px) {
  .network-chat-layout {
    grid-template-columns: 1fr;
    height: calc(100vh - 60px);
  }

  .contacts-sidebar {
    border-right: none;
  }

  .contacts-sidebar.mobile-hidden {
    display: none;
  }

  .conversation-panel.mobile-hidden {
    display: none;
  }

  .contact-row-chevron {
    display: block;  /* Show on mobile */
  }

  /* Make back button more prominent on mobile */
  .conversation-back-btn {
    display: flex;
  }
}

/* Desktop: hide back button (optional, keep for now) */
@media (min-width: 769px) {
  .conversation-back-btn {
    /* Keep visible for now, can hide if desired */
  }
}
```

---

## 6. Implementation Steps

### Phase 1: Prepare (Non-Breaking)

1. **Add new CSS classes** to `styles.rs`
   - Add all `.network-chat-layout` styles
   - Add all `.contact-row` styles
   - Add all `.empty-conversation` styles
   - Add mobile responsive rules
   - This is safe - unused CSS has no effect

2. **Add new structs** to `network.rs`
   - Add `ContactWithPreview` struct
   - Add `SelectedContact` struct
   - These are just type definitions, no behavior change

### Phase 2: Migrate State (Minimal Breaking)

3. **Update imports** in `network.rs`
   - Remove `MessageCompose`, `MessagesList`, `ReceivedMessage` imports
   - Add `ChatBubbleMessage`, `ConversationView` imports

4. **Replace state signals**
   - Remove `messages` and `compose_target` signals
   - Add `selected_contact`, `conversation_messages`, `conversation_loading`, `sending` signals

5. **Update contact loading effect**
   - Modify to build `ContactWithPreview` list
   - Add sorting by last message time

### Phase 3: Wire Up Conversation (Core Feature)

6. **Add conversation loading effect**
   - New `use_effect` that triggers when `selected_contact` changes
   - Loads messages from mirror and converts to `ChatBubbleMessage`

7. **Update send_message handler**
   - Change from modal-based to inline-based
   - Add optimistic update to `conversation_messages`

### Phase 4: Replace UI (Breaking Change)

8. **Replace RSX layout**
   - Remove old `network-grid` layout
   - Add new `network-chat-layout` with sidebar and panel
   - Remove `MessageCompose` modal at end

9. **Test thoroughly**
   - Verify contact list displays and sorts correctly
   - Verify clicking contact loads conversation
   - Verify sending message works
   - Verify back button works on mobile
   - Verify empty states display correctly

### Phase 5: Polish

10. **Add unread tracking** (if time permits)
    - Track last read sequence per contact
    - Compare with latest message sequence
    - Show unread dot

---

## 7. Engine API Wiring

### Loading Contacts with Preview

```rust
// Existing API
let contact_list: Vec<Peer> = eng.list_peer_contacts()?;

// For each contact, get their messages
for contact in &contact_list {
    if let Some(ref did_str) = contact.did {
        if let Ok(did) = did_str.parse::<Did>() {
            // Get all packets from their mirror (messages they sent us)
            let packets: Vec<PacketEnvelope> = eng.mirror_packets_since(&did, 0)?;

            // Also get packets we sent to them (from our outbox or similar)
            // TODO: May need additional API for sent messages
        }
    }
}
```

### Loading Conversation Messages

```rust
// When contact is selected
let did = selected_contact.did.parse::<Did>()?;

// Get their messages to us
let received_packets = eng.mirror_packets_since(&did, 0)?;

// Convert to ChatBubbleMessage
let messages: Vec<ChatBubbleMessage> = received_packets
    .iter()
    .filter_map(|env| {
        if env.is_global() {
            if let Ok(payload) = env.decode_global_payload() {
                if let PacketPayload::DirectMessage { content } = payload {
                    return Some(ChatBubbleMessage {
                        id: format!("{}-{}", did, env.sequence),
                        content,
                        sender_name: Some(selected_contact.name.clone()),
                        timestamp: env.timestamp,
                        is_mine: false,  // These are messages from them
                    });
                }
            }
        }
        None
    })
    .collect();

// TODO: Also need to fetch messages WE sent to THEM
// This may require:
// 1. A new engine API: eng.get_sent_messages(&did)
// 2. Or storing sent messages locally with recipient info
// 3. For now, only received messages will show
```

### Sending a Message

```rust
// Create payload
let payload = PacketPayload::DirectMessage {
    content: message_content
};

// Address to specific contact (not broadcast)
let address = PacketAddress::Individual(contact_did.parse::<Did>()?);

// Send via engine
let sequence = eng.create_and_broadcast_packet(payload, address).await?;

// Optimistically add to UI
conversation_messages.write().push(ChatBubbleMessage {
    id: format!("sent-{}", sequence),
    content: message_content,
    sender_name: None,
    timestamp: chrono::Utc::now().timestamp_millis(),
    is_mine: true,
});
```

### Real-Time Updates

```rust
// Existing: Subscribe to contact events
let mut event_rx = eng.subscribe_contact_events().await?;

while let Ok(event) = event_rx.recv().await {
    match event {
        ContactEvent::ProfileUpdated { did } => {
            // Refresh contacts list
            // If this is the selected contact, also refresh conversation
            if selected_contact().map(|c| c.did == did.to_string()).unwrap_or(false) {
                // Trigger conversation reload
            }
        }
        _ => {}
    }
}

// Existing: 5-second polling fallback
loop {
    tokio::time::sleep(Duration::from_secs(5)).await;

    if let Some(ref contact) = selected_contact() {
        // Refresh current conversation
        // Only fetch new messages (since last sequence)
    }
}
```

---

## 8. Known Limitations / Future Work

1. **Sent Messages Display**: Current implementation may only show received messages. Need to either:
   - Add engine API to retrieve sent messages by recipient
   - Store sent messages locally with recipient DID

2. **Unread Tracking**: The `unread_count` is always 0. Need to:
   - Track last read sequence per contact in local storage
   - Compare with latest message sequence

3. **Message Ordering**: Messages may need proper interleaving of sent/received based on timestamp

4. **Scroll to Bottom**: May need JavaScript interop or platform-specific solution to auto-scroll to latest message

---

## 9. Success Criteria

- [ ] Contacts sidebar displays on left (30% width)
- [ ] Contacts sorted by last message time (most recent first)
- [ ] Clicking contact loads conversation in right panel (70% width)
- [ ] Conversation shows message history with chat bubbles
- [ ] Sent messages appear aligned right (cyan background)
- [ ] Received messages appear aligned left (dark background)
- [ ] Message input at bottom of conversation
- [ ] Sending message via Enter key works
- [ ] Back button returns to contact list on mobile
- [ ] Empty state shows when no contact selected
- [ ] Mobile: contacts and conversation are full-width (stacked)
- [ ] Mobile: only one panel visible at a time
- [ ] No regression in existing functionality

---

## 10. Commit Strategy

```
1. feat(network): add CSS for chat layout redesign
   - Add network-chat-layout styles
   - Add contact-row styles
   - Add empty-conversation styles
   - Add mobile responsive rules

2. refactor(network): add data structures for chat redesign
   - Add ContactWithPreview struct
   - Add SelectedContact struct
   - Update imports

3. feat(network): implement side-by-side chat layout
   - Replace two-column grid with sidebar + panel
   - Wire up contact selection
   - Wire up conversation loading
   - Wire up message sending
   - Remove MessageCompose modal

4. fix(network): polish and responsive fixes
   - Fix any mobile layout issues
   - Adjust spacing/sizing as needed
```

---

## Handoff

To begin implementation, run:

```
/start-work network-chat-redesign
```

This plan provides all the details needed for direct implementation. The worker should follow the implementation steps in order, testing after each phase.
