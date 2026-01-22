# Synchronicity Engine - Run Logs

**Session:** 2026-01-22T10:41:27.959Z to 2026-01-22T10:41:44.036Z
**Instances:** scenario-mesh-3, peace, love, joy

## Statistics

| Level | Count |
|-------|-------|
| Total | 445 |
| INFO  | 445 |
| WARN  | 0 |
| ERROR | 0 |

---

## Instance: `joy`

Total: 131 entries (131 info, 0 warn, 0 error)

<details>
<summary>Click to expand logs</summary>

```log
10:41:28.474Z INFO syncengine_desktop - JSONL logging enabled
10:41:28.638Z INFO syncengine_desktop - Starting 'instance-joy' with data dir: "/Users/truman/Library/Application Support/syncengine-scenarios/instance-joy", screen: 1512x982, window: 504x957, total_windows: 3
10:41:29.014Z INFO syncengine_core::engine - Initializing SyncEngine
10:41:29.041Z INFO syncengine_core::blobs - Creating persistent blob manager with FsStore
10:41:29.092Z INFO syncengine_core::engine - Blob manager initialized with persistent storage
10:41:29.096Z INFO syncengine_core::engine - Creating default Private realm with sacred onboarding
10:41:29.108Z INFO syncengine_core::engine - Generating new identity
10:41:29.112Z INFO syncengine_core::engine - Deriving profile keys from identity keypair
10:41:29.124Z INFO syncengine_desktop::app - Profile name set to 'Joy'
10:41:29.124Z INFO syncengine_core::engine - Initializing contact manager
10:41:29.124Z INFO syncengine_core::engine - Initializing gossip networking
10:41:29.124Z INFO syncengine_core::engine - No endpoint secret key found, generating new one
10:41:29.128Z INFO syncengine_core::engine - Saved new endpoint secret key to storage
10:41:29.137Z INFO syncengine_core::sync::gossip - Endpoint bound
10:41:29.137Z INFO syncengine_core::sync::gossip - Gossip spawned
10:41:29.137Z INFO syncengine_core::sync::gossip - Contact protocol handler registered with shared active_topics
10:41:29.137Z INFO syncengine_core::sync::gossip - Profile protocol handler registered
10:41:29.137Z INFO syncengine_core::sync::gossip - Blob protocol handler registered for P2P image transfer
10:41:29.137Z INFO syncengine_core::sync::gossip - Router spawned
10:41:29.142Z INFO syncengine_core::sync::contact_manager - Contact auto-accept task started
10:41:29.142Z INFO syncengine_core::sync::contact_manager - Contact subscription task started
10:41:29.142Z INFO syncengine_core::engine - Contact accepted profile announcer started
10:41:29.142Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:29.142Z INFO syncengine_core::engine - Subscribed to own profile topic for broadcasting
10:41:29.142Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:29.142Z INFO syncengine_core::engine - Profile sync started on global topic
10:41:29.142Z INFO syncengine_core::engine - Profile sync listener started with P2P blob download support
10:41:29.142Z INFO syncengine_core::sync::contact_manager - Generated hybrid contact invite (v2)
10:41:29.146Z INFO syncengine_desktop::app - Bootstrap invite written to "/Users/truman/Library/Application Support/syncengine-bootstrap/joy.invite"
10:41:29.265Z INFO syncengine_core::sync::contact_handler - Received request for our own invite - will auto-accept
10:41:29.269Z INFO syncengine_core::sync::contact_handler - Received contact request, saved as IncomingPending
10:41:29.269Z INFO syncengine_core::sync::contact_manager - Auto-accepting contact request for our own invite
10:41:29.370Z INFO syncengine_core::sync::contact_manager - Accepting contact request with simplified 2-message protocol
10:41:29.376Z INFO syncengine_core::sync::contact_manager - Sent ContactAccept (simplified protocol)
10:41:29.652Z INFO syncengine_core::sync::contact_manager - Saved outgoing contact request
10:41:29.653Z INFO syncengine_core::sync::contact_manager - Sent ContactRequest
10:41:29.780Z INFO syncengine_core::sync::contact_handler - Pinned contact's profile from ContactAccept
10:41:29.784Z INFO syncengine_core::sync::contact_handler - Contact accepted and finalized with simplified protocol (keys derived locally)
10:41:29.784Z INFO syncengine_core::sync::contact_handler - Added peer to static discovery before contact topic subscription
10:41:29.784Z INFO syncengine_core::sync::contact_handler - Subscribed to contact gossip topic
10:41:29.784Z INFO syncengine_core::sync::contact_handler - Added contact topic sender to active_topics (handler)
10:41:29.784Z INFO syncengine_core::sync::contact_handler - Subscribed to contact's profile topic
10:41:29.784Z INFO syncengine_core::engine - Contact accepted, announcing profile for auto-pinning
10:41:29.784Z INFO syncengine_core::sync::contact_manager - ContactAccepted event received, ensuring topic subscription for receiving
10:41:29.784Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:29.784Z INFO syncengine_core::engine - Profile announced after contact accepted - peer should auto-pin
10:41:29.835Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:29.835Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:29.835Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:41:29.835Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:41:29.835Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:41:29.835Z INFO syncengine_core::sync::contact_manager - Successfully subscribed to contact topic for receiving messages
10:41:29.892Z INFO syncengine_core::sync::contact_manager - Pinned contact's profile from contact exchange
10:41:29.896Z INFO syncengine_core::sync::contact_manager - Finalized contact and saved to database (unified peer system)
10:41:29.896Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:29.896Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:29.896Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:41:29.896Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:41:29.896Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:41:29.896Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:29.896Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:29.896Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic for updates
10:41:29.896Z INFO syncengine_core::sync::contact_manager - Successfully auto-accepted contact request
10:41:29.896Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
10:41:29.896Z INFO syncengine_core::engine - Contact accepted, announcing profile for auto-pinning
10:41:29.896Z INFO syncengine_core::sync::contact_manager - ContactAccepted event received, ensuring topic subscription for receiving
10:41:29.896Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:29.896Z INFO syncengine_core::engine - Profile announced after contact accepted - peer should auto-pin
10:41:29.948Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:29.948Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:29.948Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:41:29.948Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:41:29.948Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:41:29.948Z INFO syncengine_core::sync::contact_manager - Successfully subscribed to contact topic for receiving messages
10:41:30.155Z INFO syncengine_desktop::app - Bootstrap: sent contact request to 'love' (attempt 1)
10:41:30.159Z INFO syncengine_core::sync::contact_manager - Saved outgoing contact request
10:41:30.160Z INFO syncengine_core::sync::contact_manager - Sent ContactRequest
10:41:30.299Z INFO syncengine_core::sync::gossip - Neighbor joined
10:41:30.299Z INFO syncengine_core::sync::gossip - Neighbor joined
10:41:30.299Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:41:30.299Z INFO syncengine_core::sync::gossip - Neighbor joined
10:41:30.299Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:41:30.300Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:41:30.307Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:41:30.307Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:41:30.315Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:41:30.315Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:41:30.319Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:41:30.319Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:41:30.323Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:41:30.327Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:41:30.331Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:41:30.654Z INFO syncengine_core::sync::contact_handler - Received contact request, saved as IncomingPending
10:41:30.663Z INFO syncengine_desktop::app - Bootstrap: sent contact request to 'peace' (attempt 1)
10:41:30.663Z INFO syncengine_desktop::app - Bootstrap: all peers connected successfully
10:41:30.663Z INFO syncengine_core::engine - Starting startup sync...
10:41:30.799Z INFO syncengine_core::sync::gossip - Neighbor joined
10:41:30.874Z INFO syncengine_core::sync::contact_manager - Auto-reconnecting to saved contacts
10:41:30.874Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:30.874Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:30.874Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:41:30.874Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:41:30.874Z INFO syncengine_core::sync::gossip - Neighbor joined
10:41:30.874Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:41:30.874Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:41:30.875Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:30.875Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:30.875Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic on reconnect
10:41:30.875Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
10:41:30.875Z INFO syncengine_core::sync::gossip - Neighbor joined
10:41:30.879Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:41:30.982Z INFO syncengine_core::engine - Profile announcement broadcast on contact topics
10:41:30.982Z INFO syncengine_core::engine - Profile announcement broadcast complete
10:41:30.982Z INFO syncengine_core::engine - Presence announced on profile and contact topics
10:41:30.982Z INFO syncengine_core::engine - Attempting startup sync with known peers
10:41:30.982Z INFO syncengine_core::engine - Startup sync complete
10:41:30.982Z INFO syncengine_desktop::app - Startup sync complete: 1 succeeded, 0 attempted, 0 skipped (backoff), jitter=209ms
10:41:30.982Z INFO syncengine_desktop::app - SyncEngine initialized with identity
10:41:30.982Z INFO syncengine_desktop::pages::landing - Returning user detected (1 realms), auto-navigating to Network
10:41:30.986Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:41:30.987Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:41:30.987Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:41:30.997Z INFO syncengine_core::engine - Loaded historical packet events from storage
10:41:32.967Z INFO syncengine_core::sync::contact_manager - Updated contact profile from per-peer topic
10:41:32.971Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:41:32.971Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:41:32.971Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:41:32.975Z INFO syncengine_core::sync::contact_manager - Updated contact profile from per-peer topic
10:41:32.975Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:41:32.975Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:41:32.975Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
```

</details>

## Instance: `love`

Total: 166 entries (166 info, 0 warn, 0 error)

<details>
<summary>Click to expand logs</summary>

```log
10:41:27.978Z INFO syncengine_desktop - JSONL logging enabled
10:41:28.144Z INFO syncengine_desktop - Starting 'instance-love' with data dir: "/Users/truman/Library/Application Support/syncengine-scenarios/instance-love", screen: 1512x982, window: 504x957, total_windows: 3
10:41:28.576Z INFO syncengine_core::engine - Initializing SyncEngine
10:41:28.607Z INFO syncengine_core::blobs - Creating persistent blob manager with FsStore
10:41:28.661Z INFO syncengine_core::engine - Blob manager initialized with persistent storage
10:41:28.666Z INFO syncengine_core::engine - Creating default Private realm with sacred onboarding
10:41:28.685Z INFO syncengine_core::engine - Generating new identity
10:41:28.691Z INFO syncengine_core::engine - Deriving profile keys from identity keypair
10:41:28.704Z INFO syncengine_desktop::app - Profile name set to 'Love'
10:41:28.704Z INFO syncengine_core::engine - Initializing contact manager
10:41:28.705Z INFO syncengine_core::engine - Initializing gossip networking
10:41:28.705Z INFO syncengine_core::engine - No endpoint secret key found, generating new one
10:41:28.709Z INFO syncengine_core::engine - Saved new endpoint secret key to storage
10:41:28.739Z INFO syncengine_core::sync::gossip - Endpoint bound
10:41:28.740Z INFO syncengine_core::sync::gossip - Gossip spawned
10:41:28.740Z INFO syncengine_core::sync::gossip - Contact protocol handler registered with shared active_topics
10:41:28.740Z INFO syncengine_core::sync::gossip - Profile protocol handler registered
10:41:28.741Z INFO syncengine_core::sync::gossip - Blob protocol handler registered for P2P image transfer
10:41:28.741Z INFO syncengine_core::sync::gossip - Router spawned
10:41:28.745Z INFO syncengine_core::sync::contact_manager - Contact auto-accept task started
10:41:28.745Z INFO syncengine_core::sync::contact_manager - Contact subscription task started
10:41:28.745Z INFO syncengine_core::engine - Contact accepted profile announcer started
10:41:28.745Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:28.746Z INFO syncengine_core::engine - Subscribed to own profile topic for broadcasting
10:41:28.746Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:28.746Z INFO syncengine_core::engine - Profile sync started on global topic
10:41:28.746Z INFO syncengine_core::engine - Profile sync listener started with P2P blob download support
10:41:28.747Z INFO syncengine_core::sync::contact_manager - Generated hybrid contact invite (v2)
10:41:28.751Z INFO syncengine_desktop::app - Bootstrap invite written to "/Users/truman/Library/Application Support/syncengine-bootstrap/love.invite"
10:41:29.257Z INFO syncengine_core::sync::contact_manager - Saved outgoing contact request
10:41:29.259Z INFO syncengine_core::sync::contact_manager - Sent ContactRequest
10:41:29.393Z INFO syncengine_core::sync::contact_handler - Pinned contact's profile from ContactAccept
10:41:29.398Z INFO syncengine_core::sync::contact_handler - Contact accepted and finalized with simplified protocol (keys derived locally)
10:41:29.398Z INFO syncengine_core::sync::contact_handler - Added peer to static discovery before contact topic subscription
10:41:29.398Z INFO syncengine_core::sync::contact_handler - Subscribed to contact gossip topic
10:41:29.398Z INFO syncengine_core::sync::contact_handler - Added contact topic sender to active_topics (handler)
10:41:29.398Z INFO syncengine_core::sync::contact_handler - Subscribed to contact's profile topic
10:41:29.398Z INFO syncengine_core::sync::contact_manager - ContactAccepted event received, ensuring topic subscription for receiving
10:41:29.398Z INFO syncengine_core::engine - Contact accepted, announcing profile for auto-pinning
10:41:29.398Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:29.398Z INFO syncengine_core::engine - Profile announced after contact accepted - peer should auto-pin
10:41:29.450Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:29.450Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:29.450Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:41:29.451Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:41:29.451Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:41:29.451Z INFO syncengine_core::sync::contact_manager - Successfully subscribed to contact topic for receiving messages
10:41:29.658Z INFO syncengine_core::sync::contact_handler - Received request for our own invite - will auto-accept
10:41:29.662Z INFO syncengine_core::sync::contact_handler - Received contact request, saved as IncomingPending
10:41:29.662Z INFO syncengine_core::sync::contact_manager - Auto-accepting contact request for our own invite
10:41:29.761Z INFO syncengine_desktop::app - Bootstrap: sent contact request to 'joy' (attempt 1)
10:41:29.764Z INFO syncengine_core::sync::contact_manager - Accepting contact request with simplified 2-message protocol
10:41:29.765Z INFO syncengine_core::sync::contact_manager - Sent ContactAccept (simplified protocol)
10:41:29.784Z INFO syncengine_core::sync::gossip - Neighbor joined
10:41:29.784Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:41:29.788Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:41:30.146Z INFO syncengine_core::sync::contact_handler - Received contact request, saved as IncomingPending
10:41:30.265Z INFO syncengine_core::sync::contact_manager - Saved outgoing contact request
10:41:30.266Z INFO syncengine_core::sync::contact_manager - Sent ContactRequest
10:41:30.283Z INFO syncengine_core::sync::contact_manager - Pinned contact's profile from contact exchange
10:41:30.295Z INFO syncengine_core::sync::contact_handler - Pinned contact's profile from ContactAccept
10:41:30.299Z INFO syncengine_core::sync::contact_manager - Finalized contact and saved to database (unified peer system)
10:41:30.299Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:30.299Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:30.299Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:41:30.299Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:41:30.299Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:41:30.299Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:30.299Z INFO syncengine_core::sync::gossip - Neighbor joined
10:41:30.299Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:30.299Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:41:30.299Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic for updates
10:41:30.299Z INFO syncengine_core::sync::contact_manager - Successfully auto-accepted contact request
10:41:30.300Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
10:41:30.300Z INFO syncengine_core::sync::contact_manager - ContactAccepted event received, ensuring topic subscription for receiving
10:41:30.300Z INFO syncengine_core::engine - Contact accepted, announcing profile for auto-pinning
10:41:30.300Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:30.303Z INFO syncengine_core::sync::contact_handler - Contact accepted and finalized with simplified protocol (keys derived locally)
10:41:30.303Z INFO syncengine_core::engine - Profile announced after contact accepted - peer should auto-pin
10:41:30.307Z INFO syncengine_core::sync::contact_handler - Added peer to static discovery before contact topic subscription
10:41:30.311Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:41:30.311Z INFO syncengine_core::sync::contact_handler - Subscribed to contact gossip topic
10:41:30.311Z INFO syncengine_core::sync::contact_handler - Added contact topic sender to active_topics (handler)
10:41:30.311Z INFO syncengine_core::sync::contact_handler - Subscribed to contact's profile topic
10:41:30.311Z INFO syncengine_core::engine - Contact accepted, announcing profile for auto-pinning
10:41:30.312Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:30.312Z INFO syncengine_core::engine - Profile announced after contact accepted - peer should auto-pin
10:41:30.351Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:30.351Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:30.351Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:41:30.351Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:41:30.351Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:41:30.351Z INFO syncengine_core::sync::contact_manager - Successfully subscribed to contact topic for receiving messages
10:41:30.351Z INFO syncengine_core::sync::gossip - Neighbor joined
10:41:30.351Z INFO syncengine_core::sync::contact_manager - ContactAccepted event received, ensuring topic subscription for receiving
10:41:30.351Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:41:30.356Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:41:30.402Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:30.402Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:30.402Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:41:30.402Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:41:30.402Z INFO syncengine_core::sync::contact_manager - Successfully subscribed to contact topic for receiving messages
10:41:30.402Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:41:30.768Z INFO syncengine_desktop::app - Bootstrap: sent contact request to 'peace' (attempt 2)
10:41:30.768Z INFO syncengine_desktop::app - Bootstrap: all peers connected successfully
10:41:30.768Z INFO syncengine_core::engine - Starting startup sync...
10:41:30.798Z INFO syncengine_core::sync::gossip - Neighbor joined
10:41:30.798Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:41:30.802Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:41:30.852Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:41:30.860Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:41:30.963Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:41:30.963Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:41:30.963Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:41:30.968Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:41:30.972Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:41:30.976Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:41:30.987Z INFO syncengine_core::sync::contact_manager - Updated contact profile from per-peer topic
10:41:32.749Z INFO syncengine_core::sync::contact_manager - Auto-reconnecting to saved contacts
10:41:32.749Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:32.749Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:32.749Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:41:32.750Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:41:32.750Z INFO syncengine_core::sync::gossip - Neighbor joined
10:41:32.750Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:41:32.750Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:41:32.750Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:32.750Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:32.750Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic on reconnect
10:41:32.750Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
10:41:32.754Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:41:32.851Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:32.851Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:32.851Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:41:32.851Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:41:32.851Z INFO syncengine_core::sync::gossip - Neighbor joined
10:41:32.851Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:41:32.851Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:41:32.851Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:32.851Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:32.851Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic on reconnect
10:41:32.851Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
10:41:32.855Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:41:32.958Z INFO syncengine_core::engine - Profile announcement broadcast on contact topics
10:41:32.958Z INFO syncengine_core::engine - Profile announcement broadcast complete
10:41:32.958Z INFO syncengine_core::engine - Presence announced on profile and contact topics
10:41:32.958Z INFO syncengine_core::engine - Attempting startup sync with known peers
10:41:32.958Z INFO syncengine_core::engine - Startup sync complete
10:41:32.958Z INFO syncengine_desktop::app - Startup sync complete: 2 succeeded, 0 attempted, 0 skipped (backoff), jitter=1979ms
10:41:32.958Z INFO syncengine_desktop::app - SyncEngine initialized with identity
10:41:32.958Z INFO syncengine_desktop::pages::landing - Returning user detected (1 realms), auto-navigating to Network
10:41:32.961Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:41:32.963Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:41:32.967Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:41:32.971Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:41:32.971Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:41:32.971Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:41:32.979Z INFO syncengine_core::engine - Loaded historical packet events from storage
10:41:39.355Z INFO syncengine_core::engine - Opening realm
10:41:39.356Z INFO syncengine_desktop::pages::field - Loaded 5 tasks for realm Private
10:41:39.356Z INFO syncengine_desktop::pages::field - Total realms loaded: 1, total task entries: 1
10:41:39.356Z INFO syncengine_desktop::pages::field - Setting signals - realms: 1, tasks_by_realm entries: 1
10:41:39.356Z INFO syncengine_desktop::pages::field - Signals set - generation now: 1, data_loaded: true
10:41:39.356Z INFO syncengine_desktop::components::unified_field - UnifiedFieldView rendering - realms: 1, tasks_by_realm entries: 1, generation: 1
10:41:39.356Z INFO syncengine_desktop::components::unified_field - Rendering realm Private with 5 tasks
10:41:39.389Z INFO syncengine_core::engine - Loaded historical packet events from storage
```

</details>

## Instance: `peace`

Total: 123 entries (123 info, 0 warn, 0 error)

<details>
<summary>Click to expand logs</summary>

```log
10:41:28.980Z INFO syncengine_desktop - JSONL logging enabled
10:41:29.137Z INFO syncengine_desktop - Starting 'instance-peace' with data dir: "/Users/truman/Library/Application Support/syncengine-scenarios/instance-peace", screen: 1512x982, window: 504x957, total_windows: 3
10:41:29.498Z INFO syncengine_core::engine - Initializing SyncEngine
10:41:29.524Z INFO syncengine_core::blobs - Creating persistent blob manager with FsStore
10:41:29.576Z INFO syncengine_core::engine - Blob manager initialized with persistent storage
10:41:29.580Z INFO syncengine_core::engine - Creating default Private realm with sacred onboarding
10:41:29.593Z INFO syncengine_core::engine - Generating new identity
10:41:29.597Z INFO syncengine_core::engine - Deriving profile keys from identity keypair
10:41:29.609Z INFO syncengine_desktop::app - Profile name set to 'Peace'
10:41:29.609Z INFO syncengine_core::engine - Initializing contact manager
10:41:29.609Z INFO syncengine_core::engine - Initializing gossip networking
10:41:29.609Z INFO syncengine_core::engine - No endpoint secret key found, generating new one
10:41:29.613Z INFO syncengine_core::engine - Saved new endpoint secret key to storage
10:41:29.622Z INFO syncengine_core::sync::gossip - Endpoint bound
10:41:29.622Z INFO syncengine_core::sync::gossip - Gossip spawned
10:41:29.622Z INFO syncengine_core::sync::gossip - Contact protocol handler registered with shared active_topics
10:41:29.622Z INFO syncengine_core::sync::gossip - Profile protocol handler registered
10:41:29.622Z INFO syncengine_core::sync::gossip - Blob protocol handler registered for P2P image transfer
10:41:29.622Z INFO syncengine_core::sync::gossip - Router spawned
10:41:29.628Z INFO syncengine_core::sync::contact_manager - Contact auto-accept task started
10:41:29.628Z INFO syncengine_core::sync::contact_manager - Contact subscription task started
10:41:29.628Z INFO syncengine_core::engine - Contact accepted profile announcer started
10:41:29.628Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:29.628Z INFO syncengine_core::engine - Subscribed to own profile topic for broadcasting
10:41:29.628Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:29.628Z INFO syncengine_core::engine - Profile sync started on global topic
10:41:29.628Z INFO syncengine_core::engine - Profile sync listener started with P2P blob download support
10:41:29.628Z INFO syncengine_core::sync::contact_manager - Generated hybrid contact invite (v2)
10:41:29.632Z INFO syncengine_desktop::app - Bootstrap invite written to "/Users/truman/Library/Application Support/syncengine-bootstrap/peace.invite"
10:41:30.139Z INFO syncengine_core::sync::contact_manager - Saved outgoing contact request
10:41:30.140Z INFO syncengine_core::sync::contact_manager - Sent ContactRequest
10:41:30.165Z INFO syncengine_core::sync::contact_handler - Received request for our own invite - will auto-accept
10:41:30.169Z INFO syncengine_core::sync::contact_handler - Received contact request, saved as IncomingPending
10:41:30.169Z INFO syncengine_core::sync::contact_manager - Auto-accepting contact request for our own invite
10:41:30.275Z INFO syncengine_core::sync::contact_handler - Received contact request, saved as IncomingPending
10:41:30.275Z INFO syncengine_core::sync::contact_manager - Accepting contact request with simplified 2-message protocol
10:41:30.276Z INFO syncengine_core::sync::contact_manager - Sent ContactAccept (simplified protocol)
10:41:30.642Z INFO syncengine_desktop::app - Bootstrap: sent contact request to 'love' (attempt 1)
10:41:30.646Z INFO syncengine_core::sync::contact_manager - Saved outgoing contact request
10:41:30.648Z INFO syncengine_core::sync::contact_manager - Sent ContactRequest
10:41:30.794Z INFO syncengine_core::sync::contact_manager - Pinned contact's profile from contact exchange
10:41:30.798Z INFO syncengine_core::sync::contact_manager - Finalized contact and saved to database (unified peer system)
10:41:30.798Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:30.798Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:30.798Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:41:30.798Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:41:30.798Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:30.798Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:30.798Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:41:30.798Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic for updates
10:41:30.798Z INFO syncengine_core::sync::contact_manager - Successfully auto-accepted contact request
10:41:30.798Z INFO syncengine_core::sync::contact_manager - ContactAccepted event received, ensuring topic subscription for receiving
10:41:30.798Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
10:41:30.798Z INFO syncengine_core::engine - Contact accepted, announcing profile for auto-pinning
10:41:30.798Z INFO syncengine_core::sync::gossip - Neighbor joined
10:41:30.798Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:41:30.798Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:30.798Z INFO syncengine_core::engine - Profile announced after contact accepted - peer should auto-pin
10:41:30.798Z INFO syncengine_core::sync::gossip - Neighbor joined
10:41:30.802Z INFO syncengine_core::sync::gossip - Neighbor joined
10:41:30.806Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:41:30.851Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:30.851Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:30.851Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:41:30.851Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:41:30.851Z INFO syncengine_core::sync::contact_manager - Successfully subscribed to contact topic for receiving messages
10:41:30.851Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:41:30.851Z INFO syncengine_core::sync::gossip - Neighbor joined
10:41:30.851Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:41:30.856Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:41:31.150Z INFO syncengine_desktop::app - Bootstrap: sent contact request to 'joy' (attempt 1)
10:41:31.150Z INFO syncengine_desktop::app - Bootstrap: all peers connected successfully
10:41:31.150Z INFO syncengine_core::engine - Starting startup sync...
10:41:32.665Z INFO syncengine_core::sync::contact_manager - Auto-reconnecting to saved contacts
10:41:32.665Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:32.665Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:32.665Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:41:32.665Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:41:32.665Z INFO syncengine_core::sync::gossip - Neighbor joined
10:41:32.665Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:41:32.665Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:41:32.665Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:41:32.665Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:41:32.665Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic on reconnect
10:41:32.665Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
10:41:32.665Z INFO syncengine_core::sync::gossip - Neighbor joined
10:41:32.665Z INFO syncengine_core::sync::gossip - Neighbor joined
10:41:32.669Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:41:32.772Z INFO syncengine_core::engine - Profile announcement broadcast on contact topics
10:41:32.772Z INFO syncengine_core::engine - Profile announcement broadcast complete
10:41:32.772Z INFO syncengine_core::engine - Presence announced on profile and contact topics
10:41:32.772Z INFO syncengine_core::engine - Attempting startup sync with known peers
10:41:32.772Z INFO syncengine_core::engine - Startup sync complete
10:41:32.772Z INFO syncengine_desktop::app - Startup sync complete: 1 succeeded, 0 attempted, 0 skipped (backoff), jitter=1513ms
10:41:32.772Z INFO syncengine_desktop::app - SyncEngine initialized with identity
10:41:32.772Z INFO syncengine_desktop::pages::landing - Returning user detected (1 realms), auto-navigating to Network
10:41:32.775Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:41:32.775Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:41:32.775Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:41:32.779Z INFO syncengine_core::engine - Loaded historical packet events from storage
10:41:32.941Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:41:32.941Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:41:32.942Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:41:32.946Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:41:32.946Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:41:32.946Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:41:32.946Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:41:32.950Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:41:32.950Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:41:32.950Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:41:32.950Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:41:32.954Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:41:32.954Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:41:32.954Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:41:32.954Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:41:32.963Z INFO syncengine_core::sync::contact_manager - Updated contact profile from per-peer topic
10:41:32.967Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:41:32.971Z INFO syncengine_core::sync::contact_manager - Updated contact profile from per-peer topic
10:41:32.971Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:41:32.971Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:41:32.971Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:41:32.971Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:41:32.971Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
```

</details>

## Instance: `scenario-mesh-3`

Total: 25 entries (25 info, 0 warn, 0 error)

<details>
<summary>Click to expand logs</summary>

```log
10:41:27.959Z INFO syncengine_scenario - JSONL logging enabled for scenario
10:41:27.959Z INFO syncengine_scenario - Loading scenario: mesh-3
10:41:27.960Z INFO syncengine_scenarios::runtime - Running scenario
10:41:27.960Z INFO syncengine_scenarios::runtime - Parsed instance names from scenario
10:41:27.960Z INFO syncengine_scenarios::runtime - Starting instance launches
10:41:27.961Z INFO syncengine_scenarios::runtime - Launching instance
10:41:27.961Z INFO syncengine_scenarios::instance - Launching instance
10:41:27.961Z INFO syncengine_scenarios::runtime - Instance launch succeeded
10:41:28.466Z INFO syncengine_scenarios::runtime - Launching instance
10:41:28.467Z INFO syncengine_scenarios::instance - Launching instance
10:41:28.467Z INFO syncengine_scenarios::runtime - Instance launch succeeded
10:41:28.972Z INFO syncengine_scenarios::runtime - Launching instance
10:41:28.973Z INFO syncengine_scenarios::instance - Launching instance
10:41:28.973Z INFO syncengine_scenarios::runtime - Instance launch succeeded
10:41:29.478Z INFO syncengine_scenarios::runtime - Finished launching instances
10:41:29.482Z INFO syncengine_scenarios::runtime - Created mesh topology with auto-connect
10:41:29.482Z INFO syncengine_scenarios::runtime - Scenario running. Press Ctrl+C to stop, or quit all instances to exit.
10:41:42.026Z INFO syncengine_scenarios::instance - Instance exited
10:41:43.534Z INFO syncengine_scenarios::instance - Instance exited
10:41:44.036Z INFO syncengine_scenarios::instance - Instance exited
10:41:44.036Z INFO syncengine_scenarios::runtime - All instances have exited, shutting down scenario...
10:41:44.036Z INFO syncengine_scenario - Scenario 'mesh-3' completed
10:41:44.036Z INFO syncengine_scenarios::instance - Killing instance
10:41:44.036Z INFO syncengine_scenarios::instance - Killing instance
10:41:44.036Z INFO syncengine_scenarios::instance - Killing instance
```

</details>

---

*Generated from JSONL logs. Regenerate with: `syncengine-cli logs report`*
