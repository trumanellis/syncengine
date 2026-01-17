# Testing Contact Exchange in the App

## Quick Start: Two-Instance Test

The easiest way to test contact exchange is running two app instances side-by-side.

### 1. Build the App

```bash
cargo build --release
```

### 2. Run Two Instances Side-by-Side

**Terminal 1 (Alice - Left Side):**
```bash
cargo run --release -- --instance 1 --position left --total-windows 2
```

**Terminal 2 (Bob - Right Side):**
```bash
cargo run --release -- --instance 2 --position right --total-windows 2
```

The windows will automatically position themselves side-by-side!

---

## Complete Test Flow

### Step 1: Navigate to Profile Page

In both windows:
1. App opens to the **Realm Hub** page
2. Click the **"Profile"** button in the top navigation
3. You should see the ProfileCard with identity information

---

### Step 2: Alice Generates Invite

**In Alice's window (left):**

1. Scroll down to find the **"Generate Contact Invite"** button
2. Click it
3. A **QR code overlay** should appear with:
   - QR code (can be scanned with phone)
   - Invite code text: `sync-contact:...`
   - **"Copy Code"** button

4. Click **"Copy Code"** to copy the invite to clipboard
5. Click anywhere outside the modal to close it

**What just happened:**
- Alice generated a cryptographically signed invite containing:
  - Her DID (decentralized identifier)
  - Her profile snapshot
  - Her network address (for QUIC connection)
  - Expiry timestamp (24 hours)

---

### Step 3: Bob Receives the Invite

**In Bob's window (right):**

1. Find the **"Add New Contact"** button (should be near the invite generator)
2. Click it
3. A modal appears: **"Add New Contact"**
4. Paste Alice's invite code into the input field
   - The code should look like: `sync-contact:...`
5. Click **"Decode Invite"**

**What just happened:**
- Bob's app:
  1. Decoded the base58 invite
  2. Verified Alice's signature
  3. Checked expiry (not expired)
  4. Created a preview of Alice's profile

**Expected result:**
- Bob should see Alice's profile preview (or an error if something failed)

---

### Step 4: Bob Sends Contact Request

**Still in Bob's window:**

1. Review Alice's profile info
2. Click **"Accept"** (or similar button to send request)

**What just happened (under the hood):**
- Bob's engine:
  1. Saved pending contact (state: `OutgoingPending`)
  2. Opened QUIC connection to Alice's network address
  3. Sent `ContactRequest` message over QUIC
  4. Emitted `ContactRequestSent` event

**Expected result in Bob's window:**
- Modal closes
- **"Pending Connections"** section appears
- Shows: "Awaiting Response: Alice - invitation sent"

**Expected result in Alice's window:**
- After ~1 second (network propagation)
- **"Pending Connections"** section appears
- Shows: "Incoming Requests: Bob wants to connect"
- Shows **"Accept"** and **"Decline"** buttons

---

### Step 5: Alice Accepts the Request

**In Alice's window (left):**

1. Find the **"Pending Connections"** section
2. See incoming request: **"Bob wants to connect"**
3. Click **"Accept"**

**What just happened (under the hood):**
- Alice's engine:
  1. Updated pending state to `WaitingForMutual`
  2. Sent `ContactResponse(accepted: true)` via QUIC to Bob
  3. Derived shared `contact_topic` and `contact_key` from both DIDs
  4. Sent `ContactAccepted` with shared keys
  5. Finalized contact (moved to contacts table)
  6. Subscribed to 1:1 gossip topic for future messages
  7. Emitted `ContactAccepted` event

**Expected result in Alice's window:**
- Pending request disappears
- **"Contacts"** gallery appears with Bob's card
- Shows: **"Contacts (0 online)"** (Bob is offline initially)

**Expected result in Bob's window:**
- After ~1-2 seconds (mutual acceptance flow)
- Pending request disappears
- **"Contacts"** gallery appears with Alice's card
- Shows: **"Contacts (0 online)"**

---

### Step 6: Verify Contact Gallery

**In both windows:**

You should now see the **"Contacts"** section with a grid layout:
- Contact card with circular avatar (or placeholder)
- Contact name below avatar
- Status dot (gray = offline)
- Hover effect (card lifts slightly)

**What to check:**
- ✅ Alice sees Bob in her contacts
- ✅ Bob sees Alice in his contacts
- ✅ Both show status as "Offline" (status dot is gray)
- ✅ No pending requests remain

---

## Advanced Testing

### Test Declined Requests

1. Alice generates invite
2. Bob decodes invite and sends request
3. Alice clicks **"Decline"** instead of Accept

**Expected result:**
- Alice's pending disappears
- Bob's pending disappears (receives decline notification)
- No contacts are created

---

### Test Multiple Contacts

1. Run a **third instance** (Charlie):
   ```bash
   cargo run --release -- --name charlie --position center
   ```

2. Alice generates invite for Charlie
3. Charlie sends request
4. Alice accepts

**Expected result:**
- Alice now has 2 contacts (Bob and Charlie)
- Gallery shows both in grid layout

---

### Test Invite Expiry

1. Generate an invite with short expiry:
   ```rust
   // In generate_invite_button.rs, change:
   eng.generate_contact_invite(24).await  // 24 hours
   // to:
   eng.generate_contact_invite(1).await   // 1 hour (for faster testing)
   ```

2. Wait for expiry (or manually advance system time)
3. Try to decode the expired invite

**Expected result:**
- Decode fails with error: "Invite expired"

---

## Debugging Tips

### Enable Debug Logging

Run with debug logs to see network activity:

```bash
RUST_LOG=debug cargo run --release -- --instance 1 --position left --total-windows 2
```

Look for:
- `ContactManager::generate_invite` - Invite generation
- `ContactManager::send_contact_request` - QUIC connection attempt
- `ContactManager::accept_contact_request` - Mutual acceptance
- `GossipSync::subscribe` - Topic subscription
- `contact_handler::handle_contact_request` - Incoming message

### Check Storage

Each instance stores data in a separate directory:

```bash
# Instance 1
ls ~/Library/Application\ Support/syncengine/instance-1/

# Instance 2
ls ~/Library/Application\ Support/syncengine/instance-2/

# View contacts in database (requires redb tool or custom query)
```

### Common Issues

**Issue: "Connecting to ourself is not supported"**
- **Cause**: Trying to use the same invite in the same instance
- **Fix**: Generate invite in Alice, decode in Bob (different instances)

**Issue: "No addressing information available"**
- **Cause**: Network not started or invalid node address
- **Fix**: Ensure `start_networking()` is called on app startup

**Issue: Pending request never appears**
- **Cause**: Network propagation delay or QUIC connection failed
- **Fix**:
  1. Check firewall settings
  2. Wait longer (up to 2-3 seconds)
  3. Check logs for connection errors

**Issue: Contacts show as "Offline" always**
- **Expected behavior**: Contacts start as Offline
- **Future enhancement**: Real-time online/offline detection via gossip

---

## What You're Testing

### Network Layer
- ✅ QUIC connection establishment
- ✅ Message serialization/deserialization
- ✅ Signature verification
- ✅ Network propagation timing

### Storage Layer
- ✅ Pending contacts saved/loaded correctly
- ✅ State transitions (OutgoingPending → WaitingForMutual → MutuallyAccepted)
- ✅ Contact finalization (pending → contacts table)

### UI Layer
- ✅ Components render correctly
- ✅ Event handlers trigger engine methods
- ✅ Reactive updates (pending appears/disappears)
- ✅ Error messages display

### Crypto Layer
- ✅ Invite signature creation/verification
- ✅ Shared key derivation (deterministic)
- ✅ Topic derivation (deterministic)

---

## Test Success Criteria

A successful end-to-end test has these outcomes:

1. ✅ Alice generates invite (shows QR + code)
2. ✅ Bob decodes invite (no errors)
3. ✅ Bob sends request (appears in his pending)
4. ✅ Alice receives request (appears in her pending after ~1 sec)
5. ✅ Alice accepts (her pending disappears)
6. ✅ Bob sees acceptance (his pending disappears after ~1-2 sec)
7. ✅ Both have each other in contacts gallery
8. ✅ Contacts show correct names and avatars
9. ✅ No crashes, no error messages

If all these pass, **your contact exchange system works end-to-end!** 🎉

---

## Next Steps

After verifying basic flow works:

1. **Test edge cases**:
   - Decline flow
   - Multiple simultaneous requests
   - Expired invites
   - Revoked invites

2. **Test persistence**:
   - Close both apps
   - Reopen
   - Verify contacts still appear

3. **Test real devices**:
   - Scan QR code with phone
   - Test over different networks (WiFi, cellular)
   - Test NAT traversal (different network segments)

4. **Performance testing**:
   - Add 10+ contacts
   - Verify gallery renders smoothly
   - Check memory usage

---

## Quick Reference

### Command Shortcuts

```bash
# Two instances side-by-side (recommended)
cargo run --release -- --instance 1 --position left --total-windows 2
cargo run --release -- --instance 2 --position right --total-windows 2

# Named instances (for clarity)
cargo run --release -- --name alice --position left --total-windows 2
cargo run --release -- --name bob --position right --total-windows 2

# Three instances (split screen)
cargo run --release -- --instance 1 --position left --total-windows 3
cargo run --release -- --instance 2 --position center --total-windows 3
cargo run --release -- --instance 3 --position right --total-windows 3

# With debug logging
RUST_LOG=debug cargo run --release -- --instance 1 --position left --total-windows 2
```

### Data Directory Locations

- **macOS**: `~/Library/Application Support/syncengine/instance-<N>/`
- **Linux**: `~/.local/share/syncengine/instance-<N>/`
- **Windows**: `%APPDATA%\syncengine\instance-<N>\`

---

## Screenshots to Expect

1. **Profile Page**: ProfileCard with identity + buttons below
2. **Generate Invite**: QR code overlay with invite code
3. **Decode Modal**: Input field + "Decode Invite" button
4. **Pending Requests**: Section with incoming/outgoing lists
5. **Contacts Gallery**: Grid of circular avatar cards
6. **Empty State**: "No contacts yet" message + hint

All with the **cyber-mystical terminal aesthetic**: black background, gold headers, cyan accents, sacred geometry patterns.
