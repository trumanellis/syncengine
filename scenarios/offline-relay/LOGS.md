# Synchronicity Engine - Run Logs

**Session:** 2026-01-22T09:42:36.288Z to 2026-01-22T09:42:50.638Z
**Instances:** peace, scenario-offline-relay, joy, love

## Statistics

| Level | Count |
|-------|-------|
| Total | 426 |
| INFO  | 425 |
| WARN  | 1 |
| ERROR | 0 |

## Warnings

- **[love]** `syncengine_desktop::app` - Message watcher: could not resolve 'peace': No contact found with name 'peace'. Available contacts: ["Joy"]

---

## Instance: `joy`

Total: 176 entries (176 info, 0 warn, 0 error)

<details>
<summary>Click to expand logs</summary>

```log
09:42:36.805Z INFO syncengine_desktop - JSONL logging enabled
09:42:36.942Z INFO syncengine_desktop - Starting 'instance-joy' with data dir: "/Users/truman/Library/Application Support/syncengine-scenarios/instance-joy", screen: 1512x982, window: 504x957, total_windows: 3
09:42:37.290Z INFO syncengine_core::engine - Initializing SyncEngine
09:42:37.319Z INFO syncengine_core::blobs - Creating persistent blob manager with FsStore
09:42:37.372Z INFO syncengine_core::engine - Blob manager initialized with persistent storage
09:42:37.376Z INFO syncengine_core::engine - Creating default Private realm with sacred onboarding
09:42:37.390Z INFO syncengine_core::engine - Generating new identity
09:42:37.394Z INFO syncengine_core::engine - Deriving profile keys from identity keypair
09:42:37.406Z INFO syncengine_desktop::app - Profile name set to 'Joy'
09:42:37.406Z INFO syncengine_core::engine - Initializing contact manager
09:42:37.406Z INFO syncengine_core::engine - Initializing gossip networking
09:42:37.406Z INFO syncengine_core::engine - No endpoint secret key found, generating new one
09:42:37.410Z INFO syncengine_core::engine - Saved new endpoint secret key to storage
09:42:37.418Z INFO syncengine_core::sync::gossip - Endpoint bound
09:42:37.418Z INFO syncengine_core::sync::gossip - Gossip spawned
09:42:37.418Z INFO syncengine_core::sync::gossip - Contact protocol handler registered with shared active_topics
09:42:37.418Z INFO syncengine_core::sync::gossip - Profile protocol handler registered
09:42:37.418Z INFO syncengine_core::sync::gossip - Blob protocol handler registered for P2P image transfer
09:42:37.418Z INFO syncengine_core::sync::gossip - Router spawned
09:42:37.423Z INFO syncengine_core::sync::contact_manager - Contact auto-accept task started
09:42:37.423Z INFO syncengine_core::sync::contact_manager - Contact subscription task started
09:42:37.423Z INFO syncengine_core::engine - Contact accepted profile announcer started
09:42:37.423Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:37.423Z INFO syncengine_core::engine - Subscribed to own profile topic for broadcasting
09:42:37.423Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:37.423Z INFO syncengine_core::engine - Profile sync started on global topic
09:42:37.423Z INFO syncengine_core::engine - Profile sync listener started with P2P blob download support
09:42:37.423Z INFO syncengine_core::sync::contact_manager - Generated hybrid contact invite (v2)
09:42:37.428Z INFO syncengine_desktop::app - Bootstrap invite written to "/Users/truman/Library/Application Support/syncengine-bootstrap/joy.invite"
09:42:37.934Z INFO syncengine_core::sync::contact_manager - Saved outgoing contact request
09:42:37.935Z INFO syncengine_core::sync::contact_manager - Sent ContactRequest
09:42:37.938Z INFO syncengine_core::sync::contact_handler - Received request for our own invite - will auto-accept
09:42:37.950Z INFO syncengine_core::sync::contact_handler - Received contact request, saved as IncomingPending
09:42:37.950Z INFO syncengine_core::sync::contact_manager - Auto-accepting contact request for our own invite
09:42:38.052Z INFO syncengine_core::sync::contact_manager - Accepting contact request with simplified 2-message protocol
09:42:38.054Z INFO syncengine_core::sync::contact_manager - Sent ContactAccept (simplified protocol)
09:42:38.084Z INFO syncengine_core::sync::contact_handler - Pinned contact's profile from ContactAccept
09:42:38.092Z INFO syncengine_core::sync::contact_handler - Contact accepted and finalized with simplified protocol (keys derived locally)
09:42:38.093Z INFO syncengine_core::sync::contact_handler - Added peer to static discovery before contact topic subscription
09:42:38.093Z INFO syncengine_core::sync::contact_handler - Subscribed to contact gossip topic
09:42:38.093Z INFO syncengine_core::sync::contact_handler - Added contact topic sender to active_topics (handler)
09:42:38.093Z INFO syncengine_core::sync::contact_handler - Subscribed to contact's profile topic
09:42:38.093Z INFO syncengine_core::sync::contact_manager - ContactAccepted event received, ensuring topic subscription for receiving
09:42:38.093Z INFO syncengine_core::engine - Contact accepted, announcing profile for auto-pinning
09:42:38.093Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:38.093Z INFO syncengine_core::engine - Profile announced after contact accepted - peer should auto-pin
09:42:38.144Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:38.144Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:38.144Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
09:42:38.145Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
09:42:38.145Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
09:42:38.145Z INFO syncengine_core::sync::contact_manager - Successfully subscribed to contact topic for receiving messages
09:42:38.145Z INFO syncengine_core::sync::gossip - Neighbor joined
09:42:38.145Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
09:42:38.150Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
09:42:38.150Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
09:42:38.158Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
09:42:38.440Z INFO syncengine_desktop::app - Bootstrap: sent contact request to 'love' (attempt 1)
09:42:38.450Z INFO syncengine_core::sync::contact_manager - Saved outgoing contact request
09:42:38.452Z INFO syncengine_core::sync::contact_manager - Sent ContactRequest
09:42:38.458Z INFO syncengine_core::sync::contact_handler - Received contact request, saved as IncomingPending
09:42:38.580Z INFO syncengine_core::sync::contact_manager - Pinned contact's profile from contact exchange
09:42:38.588Z INFO syncengine_core::sync::contact_manager - Finalized contact and saved to database (unified peer system)
09:42:38.588Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:38.588Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:38.588Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
09:42:38.589Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
09:42:38.589Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
09:42:38.591Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:38.592Z INFO syncengine_core::sync::gossip - Neighbor joined
09:42:38.592Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:38.592Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
09:42:38.592Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic for updates
09:42:38.592Z INFO syncengine_core::sync::contact_manager - Successfully auto-accepted contact request
09:42:38.592Z INFO syncengine_core::engine - Contact accepted, announcing profile for auto-pinning
09:42:38.592Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
09:42:38.592Z INFO syncengine_core::sync::contact_manager - ContactAccepted event received, ensuring topic subscription for receiving
09:42:38.592Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:38.592Z INFO syncengine_core::engine - Profile announced after contact accepted - peer should auto-pin
09:42:38.596Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
09:42:38.613Z INFO syncengine_core::sync::contact_handler - Pinned contact's profile from ContactAccept
09:42:38.617Z INFO syncengine_core::sync::contact_handler - Contact accepted and finalized with simplified protocol (keys derived locally)
09:42:38.617Z INFO syncengine_core::sync::contact_handler - Added peer to static discovery before contact topic subscription
09:42:38.618Z INFO syncengine_core::sync::contact_handler - Subscribed to contact gossip topic
09:42:38.618Z INFO syncengine_core::sync::contact_handler - Added contact topic sender to active_topics (handler)
09:42:38.618Z INFO syncengine_core::sync::contact_handler - Subscribed to contact's profile topic
09:42:38.618Z INFO syncengine_core::engine - Contact accepted, announcing profile for auto-pinning
09:42:38.618Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:38.618Z INFO syncengine_core::engine - Profile announced after contact accepted - peer should auto-pin
09:42:38.644Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:38.644Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:38.644Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
09:42:38.645Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
09:42:38.645Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
09:42:38.645Z INFO syncengine_core::sync::contact_manager - Successfully subscribed to contact topic for receiving messages
09:42:38.645Z INFO syncengine_core::sync::gossip - Neighbor joined
09:42:38.645Z INFO syncengine_core::sync::contact_manager - ContactAccepted event received, ensuring topic subscription for receiving
09:42:38.645Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
09:42:38.650Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
09:42:38.696Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:38.696Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:38.696Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
09:42:38.697Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
09:42:38.697Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
09:42:38.697Z INFO syncengine_core::sync::contact_manager - Successfully subscribed to contact topic for receiving messages
09:42:38.956Z INFO syncengine_desktop::app - Bootstrap: sent contact request to 'peace' (attempt 1)
09:42:38.956Z INFO syncengine_desktop::app - Bootstrap: all peers connected successfully
09:42:38.956Z INFO syncengine_core::engine - Starting startup sync...
09:42:39.005Z INFO syncengine_core::sync::contact_manager - Auto-reconnecting to saved contacts
09:42:39.005Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:39.005Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:39.005Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
09:42:39.006Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
09:42:39.006Z INFO syncengine_core::sync::gossip - Neighbor joined
09:42:39.006Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
09:42:39.006Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
09:42:39.006Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:39.006Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:39.006Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic on reconnect
09:42:39.006Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
09:42:39.011Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
09:42:39.109Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:39.110Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:39.110Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
09:42:39.110Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
09:42:39.111Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
09:42:39.115Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:39.115Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:39.115Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic on reconnect
09:42:39.115Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
09:42:39.115Z INFO syncengine_core::sync::gossip - Neighbor joined
09:42:39.115Z INFO syncengine_core::sync::gossip - Neighbor joined
09:42:39.115Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
09:42:39.115Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
09:42:39.120Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
09:42:39.120Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
09:42:39.130Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
09:42:39.130Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
09:42:39.135Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
09:42:39.139Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
09:42:39.222Z INFO syncengine_core::engine - Profile announcement broadcast on contact topics
09:42:39.222Z INFO syncengine_core::engine - Profile announcement broadcast complete
09:42:39.222Z INFO syncengine_core::engine - Presence announced on profile and contact topics
09:42:39.222Z INFO syncengine_core::engine - Attempting startup sync with known peers
09:42:39.222Z INFO syncengine_core::engine - Startup sync complete
09:42:39.222Z INFO syncengine_desktop::app - Startup sync complete: 2 succeeded, 0 attempted, 0 skipped (backoff), jitter=47ms
09:42:39.222Z INFO syncengine_desktop::app - SyncEngine initialized with identity
09:42:39.222Z INFO syncengine_desktop::pages::landing - Returning user detected (1 realms), auto-navigating to Network
09:42:39.229Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
09:42:39.232Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
09:42:39.232Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
09:42:39.232Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
09:42:39.232Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
09:42:39.232Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
09:42:39.250Z INFO syncengine_core::engine - Loaded historical packet events from storage
09:42:39.623Z INFO syncengine_core::sync::contact_manager - Updated contact profile from per-peer topic
09:42:39.623Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
09:42:39.623Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
09:42:39.623Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
09:42:39.623Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
09:42:39.623Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
09:42:39.623Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
09:42:39.627Z INFO syncengine_core::sync::contact_manager - Updated contact profile from per-peer topic
09:42:39.628Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
09:42:39.628Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
09:42:39.628Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
09:42:39.628Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
09:42:39.628Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
09:42:39.628Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
09:42:40.620Z INFO syncengine_core::sync::contact_manager - Updated contact profile from per-peer topic
09:42:40.621Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
09:42:40.621Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
09:42:40.621Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
09:42:40.621Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
09:42:40.621Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
09:42:40.621Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
```

</details>

## Instance: `love`

Total: 121 entries (120 info, 1 warn, 0 error)

<details>
<summary>Click to expand logs</summary>

```log
09:42:36.310Z INFO syncengine_desktop - JSONL logging enabled
09:42:36.431Z INFO syncengine_desktop - Starting 'instance-love' with data dir: "/Users/truman/Library/Application Support/syncengine-scenarios/instance-love", screen: 1512x982, window: 504x957, total_windows: 3
09:42:36.743Z INFO syncengine_core::engine - Initializing SyncEngine
09:42:36.779Z INFO syncengine_core::blobs - Creating persistent blob manager with FsStore
09:42:36.831Z INFO syncengine_core::engine - Blob manager initialized with persistent storage
09:42:36.836Z INFO syncengine_core::engine - Creating default Private realm with sacred onboarding
09:42:36.854Z INFO syncengine_core::engine - Generating new identity
09:42:36.859Z INFO syncengine_core::engine - Deriving profile keys from identity keypair
09:42:36.872Z INFO syncengine_desktop::app - Profile name set to 'Love'
09:42:36.872Z INFO syncengine_core::engine - Initializing contact manager
09:42:36.872Z INFO syncengine_core::engine - Initializing gossip networking
09:42:36.872Z INFO syncengine_core::engine - No endpoint secret key found, generating new one
09:42:36.876Z INFO syncengine_core::engine - Saved new endpoint secret key to storage
09:42:36.899Z INFO syncengine_core::sync::gossip - Endpoint bound
09:42:36.900Z INFO syncengine_core::sync::gossip - Gossip spawned
09:42:36.901Z INFO syncengine_core::sync::gossip - Contact protocol handler registered with shared active_topics
09:42:36.901Z INFO syncengine_core::sync::gossip - Profile protocol handler registered
09:42:36.901Z INFO syncengine_core::sync::gossip - Blob protocol handler registered for P2P image transfer
09:42:36.901Z INFO syncengine_core::sync::gossip - Router spawned
09:42:36.905Z INFO syncengine_core::sync::contact_manager - Contact auto-accept task started
09:42:36.905Z INFO syncengine_core::sync::contact_manager - Contact subscription task started
09:42:36.905Z INFO syncengine_core::engine - Contact accepted profile announcer started
09:42:36.906Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:36.906Z INFO syncengine_core::engine - Subscribed to own profile topic for broadcasting
09:42:36.906Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:36.906Z INFO syncengine_core::engine - Profile sync started on global topic
09:42:36.906Z INFO syncengine_core::engine - Profile sync listener started with P2P blob download support
09:42:36.906Z INFO syncengine_core::sync::contact_manager - Generated hybrid contact invite (v2)
09:42:36.910Z INFO syncengine_desktop::app - Bootstrap invite written to "/Users/truman/Library/Application Support/syncengine-bootstrap/love.invite"
09:42:37.924Z INFO syncengine_core::sync::contact_manager - Saved outgoing contact request
09:42:37.926Z INFO syncengine_core::sync::contact_manager - Sent ContactRequest
09:42:37.944Z INFO syncengine_core::sync::contact_handler - Received request for our own invite - will auto-accept
09:42:37.954Z INFO syncengine_core::sync::contact_handler - Received contact request, saved as IncomingPending
09:42:37.954Z INFO syncengine_core::sync::contact_manager - Auto-accepting contact request for our own invite
09:42:38.054Z INFO syncengine_core::sync::contact_manager - Accepting contact request with simplified 2-message protocol
09:42:38.056Z INFO syncengine_core::sync::contact_manager - Sent ContactAccept (simplified protocol)
09:42:38.079Z INFO syncengine_core::sync::contact_handler - Pinned contact's profile from ContactAccept
09:42:38.088Z INFO syncengine_core::sync::contact_handler - Contact accepted and finalized with simplified protocol (keys derived locally)
09:42:38.089Z INFO syncengine_core::sync::contact_handler - Added peer to static discovery before contact topic subscription
09:42:38.093Z INFO syncengine_core::sync::contact_handler - Subscribed to contact gossip topic
09:42:38.093Z INFO syncengine_core::sync::contact_handler - Added contact topic sender to active_topics (handler)
09:42:38.093Z INFO syncengine_core::sync::contact_handler - Subscribed to contact's profile topic
09:42:38.093Z INFO syncengine_core::engine - Contact accepted, announcing profile for auto-pinning
09:42:38.093Z INFO syncengine_core::sync::contact_manager - ContactAccepted event received, ensuring topic subscription for receiving
09:42:38.093Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:38.093Z INFO syncengine_core::engine - Profile announced after contact accepted - peer should auto-pin
09:42:38.144Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:38.144Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:38.144Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
09:42:38.145Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
09:42:38.145Z INFO syncengine_core::sync::contact_manager - Successfully subscribed to contact topic for receiving messages
09:42:38.145Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
09:42:38.145Z INFO syncengine_core::sync::gossip - Neighbor joined
09:42:38.145Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
09:42:38.150Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
09:42:38.150Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
09:42:38.154Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
09:42:38.430Z INFO syncengine_desktop::app - Bootstrap: sent contact request to 'joy' (attempt 2)
09:42:38.435Z INFO syncengine_core::sync::contact_manager - Saved outgoing contact request
09:42:38.438Z INFO syncengine_core::sync::contact_manager - Sent ContactRequest
09:42:38.583Z INFO syncengine_core::sync::contact_manager - Pinned contact's profile from contact exchange
09:42:38.591Z INFO syncengine_core::sync::contact_manager - Finalized contact and saved to database (unified peer system)
09:42:38.592Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:38.592Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:38.592Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
09:42:38.592Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
09:42:38.592Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
09:42:38.592Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:38.592Z INFO syncengine_core::sync::gossip - Neighbor joined
09:42:38.596Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:38.596Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
09:42:38.596Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic for updates
09:42:38.596Z INFO syncengine_core::sync::contact_manager - Successfully auto-accepted contact request
09:42:38.596Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
09:42:38.596Z INFO syncengine_core::engine - Contact accepted, announcing profile for auto-pinning
09:42:38.596Z INFO syncengine_core::sync::contact_manager - ContactAccepted event received, ensuring topic subscription for receiving
09:42:38.596Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:38.599Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
09:42:38.600Z INFO syncengine_core::engine - Profile announced after contact accepted - peer should auto-pin
09:42:38.647Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:38.650Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:38.650Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
09:42:38.650Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
09:42:38.650Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
09:42:38.650Z INFO syncengine_core::sync::contact_manager - Successfully subscribed to contact topic for receiving messages
09:42:38.650Z INFO syncengine_core::sync::gossip - Neighbor joined
09:42:38.650Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
09:42:38.654Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
09:42:38.941Z INFO syncengine_desktop::app - Bootstrap: sent contact request to 'peace' (attempt 2)
09:42:38.941Z INFO syncengine_desktop::app - Bootstrap: all peers connected successfully
09:42:38.941Z INFO syncengine_core::engine - Starting startup sync...
09:42:38.962Z INFO syncengine_core::sync::contact_handler - Received contact request, saved as IncomingPending
09:42:39.118Z INFO syncengine_core::sync::gossip - Neighbor joined
09:42:39.228Z INFO syncengine_core::sync::contact_manager - Updated contact profile from per-peer topic
09:42:39.510Z INFO syncengine_core::sync::contact_manager - Auto-reconnecting to saved contacts
09:42:39.510Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:39.510Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:39.510Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
09:42:39.510Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
09:42:39.510Z INFO syncengine_core::sync::gossip - Neighbor joined
09:42:39.510Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
09:42:39.511Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
09:42:39.511Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:39.511Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:39.511Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic on reconnect
09:42:39.511Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
09:42:39.511Z INFO syncengine_core::sync::gossip - Neighbor joined
09:42:39.515Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
09:42:39.618Z INFO syncengine_core::engine - Profile announcement broadcast on contact topics
09:42:39.618Z INFO syncengine_core::engine - Profile announcement broadcast complete
09:42:39.618Z INFO syncengine_core::engine - Presence announced on profile and contact topics
09:42:39.618Z INFO syncengine_core::engine - Attempting startup sync with known peers
09:42:39.618Z INFO syncengine_core::engine - Startup sync complete
09:42:39.618Z INFO syncengine_desktop::app - Startup sync complete: 1 succeeded, 0 attempted, 0 skipped (backoff), jitter=567ms
09:42:39.618Z INFO syncengine_desktop::app - SyncEngine initialized with identity
09:42:39.618Z INFO syncengine_desktop::pages::landing - Returning user detected (1 realms), auto-navigating to Network
09:42:39.622Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
09:42:39.623Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
09:42:39.623Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
09:42:39.629Z INFO syncengine_core::engine - Loaded historical packet events from storage
09:42:50.638Z WARN syncengine_desktop::app - Message watcher: could not resolve 'peace': No contact found with name 'peace'. Available contacts: ["Joy"]
```

</details>

## Instance: `peace`

Total: 105 entries (105 info, 0 warn, 0 error)

<details>
<summary>Click to expand logs</summary>

```log
09:42:37.316Z INFO syncengine_desktop - JSONL logging enabled
09:42:37.446Z INFO syncengine_desktop - Starting 'instance-peace' with data dir: "/Users/truman/Library/Application Support/syncengine-scenarios/instance-peace", screen: 1512x982, window: 504x957, total_windows: 3
09:42:37.776Z INFO syncengine_core::engine - Initializing SyncEngine
09:42:37.809Z INFO syncengine_core::blobs - Creating persistent blob manager with FsStore
09:42:37.862Z INFO syncengine_core::engine - Blob manager initialized with persistent storage
09:42:37.867Z INFO syncengine_core::engine - Creating default Private realm with sacred onboarding
09:42:37.881Z INFO syncengine_core::engine - Generating new identity
09:42:37.886Z INFO syncengine_core::engine - Deriving profile keys from identity keypair
09:42:37.899Z INFO syncengine_desktop::app - Profile name set to 'Peace'
09:42:37.899Z INFO syncengine_core::engine - Initializing contact manager
09:42:37.899Z INFO syncengine_core::engine - Initializing gossip networking
09:42:37.899Z INFO syncengine_core::engine - No endpoint secret key found, generating new one
09:42:37.902Z INFO syncengine_core::engine - Saved new endpoint secret key to storage
09:42:37.912Z INFO syncengine_core::sync::gossip - Endpoint bound
09:42:37.912Z INFO syncengine_core::sync::gossip - Gossip spawned
09:42:37.912Z INFO syncengine_core::sync::gossip - Contact protocol handler registered with shared active_topics
09:42:37.912Z INFO syncengine_core::sync::gossip - Profile protocol handler registered
09:42:37.912Z INFO syncengine_core::sync::gossip - Blob protocol handler registered for P2P image transfer
09:42:37.912Z INFO syncengine_core::sync::gossip - Router spawned
09:42:37.920Z INFO syncengine_core::sync::contact_manager - Contact auto-accept task started
09:42:37.920Z INFO syncengine_core::sync::contact_manager - Contact subscription task started
09:42:37.920Z INFO syncengine_core::engine - Contact accepted profile announcer started
09:42:37.920Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:37.920Z INFO syncengine_core::engine - Subscribed to own profile topic for broadcasting
09:42:37.920Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:37.920Z INFO syncengine_core::engine - Profile sync started on global topic
09:42:37.920Z INFO syncengine_core::engine - Profile sync listener started with P2P blob download support
09:42:37.920Z INFO syncengine_core::sync::contact_manager - Generated hybrid contact invite (v2)
09:42:37.929Z INFO syncengine_desktop::app - Bootstrap invite written to "/Users/truman/Library/Application Support/syncengine-bootstrap/peace.invite"
09:42:38.439Z INFO syncengine_core::sync::contact_manager - Saved outgoing contact request
09:42:38.441Z INFO syncengine_core::sync::contact_manager - Sent ContactRequest
09:42:38.444Z INFO syncengine_core::sync::contact_handler - Received request for our own invite - will auto-accept
09:42:38.454Z INFO syncengine_core::sync::contact_handler - Received contact request, saved as IncomingPending
09:42:38.455Z INFO syncengine_core::sync::contact_manager - Auto-accepting contact request for our own invite
09:42:38.462Z INFO syncengine_core::sync::contact_handler - Received contact request, saved as IncomingPending
09:42:38.556Z INFO syncengine_core::sync::contact_manager - Accepting contact request with simplified 2-message protocol
09:42:38.589Z INFO syncengine_core::sync::contact_manager - Sent ContactAccept (simplified protocol)
09:42:38.946Z INFO syncengine_desktop::app - Bootstrap: sent contact request to 'joy' (attempt 1)
09:42:38.951Z INFO syncengine_core::sync::contact_manager - Saved outgoing contact request
09:42:38.954Z INFO syncengine_core::sync::contact_manager - Sent ContactRequest
09:42:39.110Z INFO syncengine_core::sync::contact_manager - Pinned contact's profile from contact exchange
09:42:39.115Z INFO syncengine_core::sync::contact_manager - Finalized contact and saved to database (unified peer system)
09:42:39.115Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:39.115Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:39.115Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
09:42:39.115Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
09:42:39.115Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
09:42:39.115Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:39.116Z INFO syncengine_core::sync::gossip - Neighbor joined
09:42:39.116Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:39.116Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
09:42:39.116Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic for updates
09:42:39.116Z INFO syncengine_core::sync::contact_manager - Successfully auto-accepted contact request
09:42:39.116Z INFO syncengine_core::sync::contact_manager - ContactAccepted event received, ensuring topic subscription for receiving
09:42:39.116Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
09:42:39.116Z INFO syncengine_core::engine - Contact accepted, announcing profile for auto-pinning
09:42:39.116Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:39.116Z INFO syncengine_core::sync::gossip - Neighbor joined
09:42:39.120Z INFO syncengine_core::engine - Profile announced after contact accepted - peer should auto-pin
09:42:39.125Z INFO syncengine_core::sync::gossip - Neighbor joined
09:42:39.125Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
09:42:39.166Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:39.166Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:39.166Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
09:42:39.167Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
09:42:39.167Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
09:42:39.167Z INFO syncengine_core::sync::contact_manager - Successfully subscribed to contact topic for receiving messages
09:42:39.167Z INFO syncengine_core::sync::gossip - Neighbor joined
09:42:39.167Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
09:42:39.172Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
09:42:39.232Z INFO syncengine_core::sync::contact_manager - Updated contact profile from per-peer topic
09:42:39.309Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
09:42:39.309Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
09:42:39.315Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
09:42:39.319Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
09:42:39.456Z INFO syncengine_desktop::app - Bootstrap: sent contact request to 'love' (attempt 1)
09:42:39.457Z INFO syncengine_desktop::app - Bootstrap: all peers connected successfully
09:42:39.457Z INFO syncengine_core::engine - Starting startup sync...
09:42:40.506Z INFO syncengine_core::sync::contact_manager - Auto-reconnecting to saved contacts
09:42:40.506Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:40.506Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:40.506Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
09:42:40.506Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
09:42:40.506Z INFO syncengine_core::sync::gossip - Neighbor joined
09:42:40.506Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
09:42:40.506Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
09:42:40.507Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
09:42:40.507Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
09:42:40.507Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic on reconnect
09:42:40.507Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
09:42:40.507Z INFO syncengine_core::sync::gossip - Neighbor joined
09:42:40.507Z INFO syncengine_core::sync::gossip - Neighbor joined
09:42:40.511Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
09:42:40.614Z INFO syncengine_core::engine - Profile announcement broadcast on contact topics
09:42:40.614Z INFO syncengine_core::engine - Profile announcement broadcast complete
09:42:40.614Z INFO syncengine_core::engine - Presence announced on profile and contact topics
09:42:40.615Z INFO syncengine_core::engine - Attempting startup sync with known peers
09:42:40.615Z INFO syncengine_core::engine - Startup sync complete
09:42:40.615Z INFO syncengine_desktop::app - Startup sync complete: 1 succeeded, 0 attempted, 0 skipped (backoff), jitter=1047ms
09:42:40.615Z INFO syncengine_desktop::app - SyncEngine initialized with identity
09:42:40.615Z INFO syncengine_desktop::pages::landing - Returning user detected (1 realms), auto-navigating to Network
09:42:40.620Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
09:42:40.620Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
09:42:40.620Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
09:42:40.627Z INFO syncengine_core::engine - Loaded historical packet events from storage
```

</details>

## Instance: `scenario-offline-relay`

Total: 24 entries (24 info, 0 warn, 0 error)

<details>
<summary>Click to expand logs</summary>

```log
09:42:36.288Z INFO syncengine_scenario - JSONL logging enabled for scenario
09:42:36.289Z INFO syncengine_scenario - Loading scenario: offline-relay
09:42:36.291Z INFO syncengine_scenarios::runtime - Running scenario
09:42:36.291Z INFO syncengine_scenarios::runtime - Parsed instance names from scenario
09:42:36.291Z INFO syncengine_scenarios::runtime - Starting instance launches
09:42:36.291Z INFO syncengine_scenarios::runtime - Launching instance
09:42:36.291Z INFO syncengine_scenarios::instance - Launching instance
09:42:36.292Z INFO syncengine_scenarios::runtime - Instance launch succeeded
09:42:36.795Z INFO syncengine_scenarios::runtime - Launching instance
09:42:36.798Z INFO syncengine_scenarios::instance - Launching instance
09:42:36.798Z INFO syncengine_scenarios::runtime - Instance launch succeeded
09:42:37.303Z INFO syncengine_scenarios::runtime - Launching instance
09:42:37.307Z INFO syncengine_scenarios::instance - Launching instance
09:42:37.308Z INFO syncengine_scenarios::runtime - Instance launch succeeded
09:42:37.813Z INFO syncengine_scenarios::runtime - Finished launching instances
09:42:37.816Z INFO syncengine_scenarios::runtime - Created mesh topology with auto-connect
09:42:37.816Z INFO scenario - === Offline Relay Test (Full Mesh) ===
09:42:37.816Z INFO scenario - Topology: love <-> joy <-> peace <-> love
09:42:37.816Z INFO scenario - All nodes start connected to each other
09:42:37.816Z INFO scenario - 
09:42:45.818Z INFO scenario - Phase 1: Mesh established, killing Peace...
09:42:45.818Z INFO syncengine_scenarios::instance - Killing instance
09:42:49.818Z INFO scenario - Phase 2: Love sending message to Peace (who is offline)...
09:42:49.820Z INFO syncengine_scenarios::api - Wrote message instruction to love.sendmsg
```

</details>

---

*Generated from JSONL logs. Regenerate with: `syncengine-cli logs report`*
