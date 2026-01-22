# Synchronicity Engine - Run Logs

**Session:** 2026-01-22T10:38:44.436Z to 2026-01-22T10:39:03.461Z
**Instances:** joy, peace, scenario-mesh-messaging, love

## Statistics

| Level | Count |
|-------|-------|
| Total | 520 |
| INFO  | 516 |
| WARN  | 4 |
| ERROR | 0 |

## Warnings

- **[joy]** `syncengine_core::sync::contact_manager` - Failed to parse relayed packet envelope
- **[joy]** `syncengine_core::sync::contact_manager` - Failed to parse relayed packet envelope
- **[joy]** `syncengine_core::sync::contact_manager` - Failed to parse relayed packet envelope
- **[joy]** `syncengine_core::sync::contact_manager` - Failed to parse relayed packet envelope

---

## Instance: `joy`

Total: 136 entries (132 info, 4 warn, 0 error)

<details>
<summary>Click to expand logs</summary>

```log
10:38:44.948Z INFO syncengine_desktop - JSONL logging enabled
10:38:45.095Z INFO syncengine_desktop - Starting 'instance-joy' with data dir: "/Users/truman/Library/Application Support/syncengine-scenarios/instance-joy", screen: 1512x982, window: 504x957, total_windows: 3
10:38:45.440Z INFO syncengine_core::engine - Initializing SyncEngine
10:38:45.469Z INFO syncengine_core::blobs - Creating persistent blob manager with FsStore
10:38:45.521Z INFO syncengine_core::engine - Blob manager initialized with persistent storage
10:38:45.525Z INFO syncengine_core::engine - Creating default Private realm with sacred onboarding
10:38:45.538Z INFO syncengine_core::engine - Generating new identity
10:38:45.543Z INFO syncengine_core::engine - Deriving profile keys from identity keypair
10:38:45.555Z INFO syncengine_desktop::app - Profile name set to 'Joy'
10:38:45.555Z INFO syncengine_core::engine - Initializing contact manager
10:38:45.555Z INFO syncengine_core::engine - Initializing gossip networking
10:38:45.555Z INFO syncengine_core::engine - No endpoint secret key found, generating new one
10:38:45.559Z INFO syncengine_core::engine - Saved new endpoint secret key to storage
10:38:45.567Z INFO syncengine_core::sync::gossip - Endpoint bound
10:38:45.567Z INFO syncengine_core::sync::gossip - Gossip spawned
10:38:45.567Z INFO syncengine_core::sync::gossip - Contact protocol handler registered with shared active_topics
10:38:45.567Z INFO syncengine_core::sync::gossip - Profile protocol handler registered
10:38:45.568Z INFO syncengine_core::sync::gossip - Blob protocol handler registered for P2P image transfer
10:38:45.568Z INFO syncengine_core::sync::gossip - Router spawned
10:38:45.573Z INFO syncengine_core::sync::contact_manager - Contact auto-accept task started
10:38:45.573Z INFO syncengine_core::sync::contact_manager - Contact subscription task started
10:38:45.573Z INFO syncengine_core::engine - Contact accepted profile announcer started
10:38:45.573Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:45.573Z INFO syncengine_core::engine - Subscribed to own profile topic for broadcasting
10:38:45.573Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:45.573Z INFO syncengine_core::engine - Profile sync started on global topic
10:38:45.573Z INFO syncengine_core::engine - Profile sync listener started with P2P blob download support
10:38:45.573Z INFO syncengine_core::sync::contact_manager - Generated hybrid contact invite (v2)
10:38:45.578Z INFO syncengine_desktop::app - Bootstrap invite written to "/Users/truman/Library/Application Support/syncengine-bootstrap/joy.invite"
10:38:45.710Z INFO syncengine_core::sync::contact_handler - Received request for our own invite - will auto-accept
10:38:45.715Z INFO syncengine_core::sync::contact_handler - Received contact request, saved as IncomingPending
10:38:45.715Z INFO syncengine_core::sync::contact_manager - Auto-accepting contact request for our own invite
10:38:45.818Z INFO syncengine_core::sync::contact_manager - Accepting contact request with simplified 2-message protocol
10:38:45.820Z INFO syncengine_core::sync::contact_manager - Sent ContactAccept (simplified protocol)
10:38:46.084Z INFO syncengine_core::sync::contact_manager - Saved outgoing contact request
10:38:46.086Z INFO syncengine_core::sync::contact_manager - Sent ContactRequest
10:38:46.236Z INFO syncengine_core::sync::contact_handler - Pinned contact's profile from ContactAccept
10:38:46.240Z INFO syncengine_core::sync::contact_handler - Contact accepted and finalized with simplified protocol (keys derived locally)
10:38:46.240Z INFO syncengine_core::sync::contact_handler - Added peer to static discovery before contact topic subscription
10:38:46.241Z INFO syncengine_core::sync::contact_handler - Subscribed to contact gossip topic
10:38:46.241Z INFO syncengine_core::sync::contact_handler - Added contact topic sender to active_topics (handler)
10:38:46.241Z INFO syncengine_core::sync::contact_handler - Subscribed to contact's profile topic
10:38:46.241Z INFO syncengine_core::sync::contact_manager - ContactAccepted event received, ensuring topic subscription for receiving
10:38:46.241Z INFO syncengine_core::engine - Contact accepted, announcing profile for auto-pinning
10:38:46.241Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:46.241Z INFO syncengine_core::engine - Profile announced after contact accepted - peer should auto-pin
10:38:46.292Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:46.292Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:46.292Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:38:46.293Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:38:46.293Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:38:46.293Z INFO syncengine_core::sync::contact_manager - Successfully subscribed to contact topic for receiving messages
10:38:46.336Z INFO syncengine_core::sync::contact_manager - Pinned contact's profile from contact exchange
10:38:46.340Z INFO syncengine_core::sync::contact_manager - Finalized contact and saved to database (unified peer system)
10:38:46.340Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:46.340Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:46.340Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:38:46.341Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:38:46.341Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:46.341Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:38:46.341Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:46.341Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic for updates
10:38:46.341Z INFO syncengine_core::sync::contact_manager - Successfully auto-accepted contact request
10:38:46.341Z INFO syncengine_core::sync::contact_manager - ContactAccepted event received, ensuring topic subscription for receiving
10:38:46.341Z INFO syncengine_core::engine - Contact accepted, announcing profile for auto-pinning
10:38:46.341Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
10:38:46.341Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:46.341Z INFO syncengine_core::engine - Profile announced after contact accepted - peer should auto-pin
10:38:46.392Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:46.392Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:46.392Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:38:46.393Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:38:46.393Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:38:46.393Z INFO syncengine_core::sync::contact_manager - Successfully subscribed to contact topic for receiving messages
10:38:46.588Z INFO syncengine_desktop::app - Bootstrap: sent contact request to 'love' (attempt 1)
10:38:46.593Z INFO syncengine_core::sync::contact_manager - Saved outgoing contact request
10:38:46.595Z INFO syncengine_core::sync::contact_manager - Sent ContactRequest
10:38:46.622Z INFO syncengine_core::sync::contact_handler - Received contact request, saved as IncomingPending
10:38:46.730Z INFO syncengine_core::sync::gossip - Neighbor joined
10:38:46.730Z INFO syncengine_core::sync::gossip - Neighbor joined
10:38:46.730Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:38:46.730Z INFO syncengine_core::sync::gossip - Neighbor joined
10:38:46.731Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:38:46.731Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:38:46.739Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:38:46.739Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:38:46.743Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:38:46.743Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:38:46.747Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:38:46.747Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:38:46.751Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:38:46.755Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:38:46.759Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:38:46.859Z INFO syncengine_core::sync::gossip - Neighbor joined
10:38:47.098Z INFO syncengine_desktop::app - Bootstrap: sent contact request to 'peace' (attempt 1)
10:38:47.098Z INFO syncengine_desktop::app - Bootstrap: all peers connected successfully
10:38:47.098Z INFO syncengine_core::engine - Starting startup sync...
10:38:48.120Z INFO syncengine_core::sync::contact_manager - Updated contact profile from per-peer topic
10:38:49.053Z INFO syncengine_core::sync::contact_manager - Auto-reconnecting to saved contacts
10:38:49.053Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:49.053Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:49.053Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:38:49.053Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:38:49.053Z INFO syncengine_core::sync::gossip - Neighbor joined
10:38:49.053Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:38:49.054Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:38:49.054Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:49.054Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:49.054Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic on reconnect
10:38:49.054Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
10:38:49.054Z INFO syncengine_core::sync::gossip - Neighbor joined
10:38:49.058Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:38:49.164Z INFO syncengine_core::engine - Profile announcement broadcast on contact topics
10:38:49.164Z INFO syncengine_core::engine - Profile announcement broadcast complete
10:38:49.164Z INFO syncengine_core::engine - Presence announced on profile and contact topics
10:38:49.164Z INFO syncengine_core::engine - Attempting startup sync with known peers
10:38:49.164Z INFO syncengine_core::engine - Startup sync complete
10:38:49.164Z INFO syncengine_desktop::app - Startup sync complete: 1 succeeded, 0 attempted, 0 skipped (backoff), jitter=1954ms
10:38:49.164Z INFO syncengine_desktop::app - SyncEngine initialized with identity
10:38:49.164Z INFO syncengine_desktop::pages::landing - Returning user detected (1 realms), auto-navigating to Network
10:38:49.168Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:49.168Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:38:49.168Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:38:49.172Z INFO syncengine_core::engine - Loaded historical packet events from storage
10:38:54.224Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:38:54.224Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:38:54.224Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:38:54.224Z INFO syncengine_core::sync::contact_manager - Received RELAY request - storing for forwarding
10:38:54.224Z INFO syncengine_core::sync::contact_manager - Received RELAY request - storing for forwarding
10:38:54.224Z INFO syncengine_core::sync::contact_manager - Received RELAY request - storing for forwarding
10:38:54.224Z WARN syncengine_core::sync::contact_manager - Failed to parse relayed packet envelope
10:38:54.224Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:38:54.224Z WARN syncengine_core::sync::contact_manager - Failed to parse relayed packet envelope
10:38:54.224Z INFO syncengine_core::sync::contact_manager - Received RELAY request - storing for forwarding
10:38:54.224Z WARN syncengine_core::sync::contact_manager - Failed to parse relayed packet envelope
10:38:54.224Z WARN syncengine_core::sync::contact_manager - Failed to parse relayed packet envelope
```

</details>

## Instance: `love`

Total: 210 entries (210 info, 0 warn, 0 error)

<details>
<summary>Click to expand logs</summary>

```log
10:38:44.457Z INFO syncengine_desktop - JSONL logging enabled
10:38:44.612Z INFO syncengine_desktop - Starting 'instance-love' with data dir: "/Users/truman/Library/Application Support/syncengine-scenarios/instance-love", screen: 1512x982, window: 504x957, total_windows: 3
10:38:45.022Z INFO syncengine_core::engine - Initializing SyncEngine
10:38:45.051Z INFO syncengine_core::blobs - Creating persistent blob manager with FsStore
10:38:45.105Z INFO syncengine_core::engine - Blob manager initialized with persistent storage
10:38:45.110Z INFO syncengine_core::engine - Creating default Private realm with sacred onboarding
10:38:45.128Z INFO syncengine_core::engine - Generating new identity
10:38:45.134Z INFO syncengine_core::engine - Deriving profile keys from identity keypair
10:38:45.151Z INFO syncengine_desktop::app - Profile name set to 'Love'
10:38:45.151Z INFO syncengine_core::engine - Initializing contact manager
10:38:45.151Z INFO syncengine_core::engine - Initializing gossip networking
10:38:45.151Z INFO syncengine_core::engine - No endpoint secret key found, generating new one
10:38:45.156Z INFO syncengine_core::engine - Saved new endpoint secret key to storage
10:38:45.182Z INFO syncengine_core::sync::gossip - Endpoint bound
10:38:45.184Z INFO syncengine_core::sync::gossip - Gossip spawned
10:38:45.184Z INFO syncengine_core::sync::gossip - Contact protocol handler registered with shared active_topics
10:38:45.184Z INFO syncengine_core::sync::gossip - Profile protocol handler registered
10:38:45.184Z INFO syncengine_core::sync::gossip - Blob protocol handler registered for P2P image transfer
10:38:45.184Z INFO syncengine_core::sync::gossip - Router spawned
10:38:45.189Z INFO syncengine_core::sync::contact_manager - Contact auto-accept task started
10:38:45.189Z INFO syncengine_core::sync::contact_manager - Contact subscription task started
10:38:45.189Z INFO syncengine_core::engine - Contact accepted profile announcer started
10:38:45.189Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:45.189Z INFO syncengine_core::engine - Subscribed to own profile topic for broadcasting
10:38:45.189Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:45.189Z INFO syncengine_core::engine - Profile sync started on global topic
10:38:45.190Z INFO syncengine_core::engine - Profile sync listener started with P2P blob download support
10:38:45.190Z INFO syncengine_core::sync::contact_manager - Generated hybrid contact invite (v2)
10:38:45.196Z INFO syncengine_desktop::app - Bootstrap invite written to "/Users/truman/Library/Application Support/syncengine-bootstrap/love.invite"
10:38:45.702Z INFO syncengine_core::sync::contact_manager - Saved outgoing contact request
10:38:45.705Z INFO syncengine_core::sync::contact_manager - Sent ContactRequest
10:38:45.838Z INFO syncengine_core::sync::contact_handler - Pinned contact's profile from ContactAccept
10:38:45.842Z INFO syncengine_core::sync::contact_handler - Contact accepted and finalized with simplified protocol (keys derived locally)
10:38:45.842Z INFO syncengine_core::sync::contact_handler - Added peer to static discovery before contact topic subscription
10:38:45.842Z INFO syncengine_core::sync::contact_handler - Subscribed to contact gossip topic
10:38:45.842Z INFO syncengine_core::sync::contact_handler - Added contact topic sender to active_topics (handler)
10:38:45.842Z INFO syncengine_core::sync::contact_handler - Subscribed to contact's profile topic
10:38:45.842Z INFO syncengine_core::engine - Contact accepted, announcing profile for auto-pinning
10:38:45.842Z INFO syncengine_core::sync::contact_manager - ContactAccepted event received, ensuring topic subscription for receiving
10:38:45.842Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:45.843Z INFO syncengine_core::engine - Profile announced after contact accepted - peer should auto-pin
10:38:45.895Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:45.895Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:45.895Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:38:45.895Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:38:45.895Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:38:45.895Z INFO syncengine_core::sync::contact_manager - Successfully subscribed to contact topic for receiving messages
10:38:46.093Z INFO syncengine_core::sync::contact_handler - Received request for our own invite - will auto-accept
10:38:46.103Z INFO syncengine_core::sync::contact_handler - Received contact request, saved as IncomingPending
10:38:46.103Z INFO syncengine_core::sync::contact_manager - Auto-accepting contact request for our own invite
10:38:46.206Z INFO syncengine_core::sync::contact_manager - Accepting contact request with simplified 2-message protocol
10:38:46.206Z INFO syncengine_desktop::app - Bootstrap: sent contact request to 'joy' (attempt 1)
10:38:46.207Z INFO syncengine_core::sync::contact_manager - Sent ContactAccept (simplified protocol)
10:38:46.211Z INFO syncengine_core::sync::contact_manager - Saved outgoing contact request
10:38:46.212Z INFO syncengine_core::sync::contact_manager - Sent ContactRequest
10:38:46.241Z INFO syncengine_core::sync::gossip - Neighbor joined
10:38:46.241Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:38:46.245Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:38:46.355Z INFO syncengine_core::sync::contact_handler - Pinned contact's profile from ContactAccept
10:38:46.359Z INFO syncengine_core::sync::contact_handler - Contact accepted and finalized with simplified protocol (keys derived locally)
10:38:46.359Z INFO syncengine_core::sync::contact_handler - Added peer to static discovery before contact topic subscription
10:38:46.359Z INFO syncengine_core::sync::contact_handler - Subscribed to contact gossip topic
10:38:46.359Z INFO syncengine_core::sync::contact_handler - Added contact topic sender to active_topics (handler)
10:38:46.359Z INFO syncengine_core::sync::contact_handler - Subscribed to contact's profile topic
10:38:46.359Z INFO syncengine_core::sync::contact_manager - ContactAccepted event received, ensuring topic subscription for receiving
10:38:46.359Z INFO syncengine_core::engine - Contact accepted, announcing profile for auto-pinning
10:38:46.359Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:46.359Z INFO syncengine_core::engine - Profile announced after contact accepted - peer should auto-pin
10:38:46.410Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:46.410Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:46.410Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:38:46.410Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:38:46.410Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:38:46.410Z INFO syncengine_core::sync::contact_manager - Successfully subscribed to contact topic for receiving messages
10:38:46.726Z INFO syncengine_core::sync::contact_manager - Pinned contact's profile from contact exchange
10:38:46.730Z INFO syncengine_core::sync::contact_manager - Finalized contact and saved to database (unified peer system)
10:38:46.730Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:46.730Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:46.730Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:38:46.730Z INFO syncengine_desktop::app - Bootstrap: sent contact request to 'peace' (attempt 1)
10:38:46.730Z INFO syncengine_desktop::app - Bootstrap: all peers connected successfully
10:38:46.730Z INFO syncengine_core::engine - Starting startup sync...
10:38:46.730Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:38:46.730Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:38:46.730Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:46.730Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:46.730Z INFO syncengine_core::sync::gossip - Neighbor joined
10:38:46.730Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic for updates
10:38:46.730Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:38:46.730Z INFO syncengine_core::sync::contact_manager - Successfully auto-accepted contact request
10:38:46.730Z INFO syncengine_core::sync::contact_manager - ContactAccepted event received, ensuring topic subscription for receiving
10:38:46.730Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
10:38:46.730Z INFO syncengine_core::engine - Contact accepted, announcing profile for auto-pinning
10:38:46.730Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:46.731Z INFO syncengine_core::engine - Profile announced after contact accepted - peer should auto-pin
10:38:46.735Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:38:46.782Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:46.782Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:46.782Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:38:46.782Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:38:46.782Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:38:46.782Z INFO syncengine_core::sync::contact_manager - Successfully subscribed to contact topic for receiving messages
10:38:46.782Z INFO syncengine_core::sync::gossip - Neighbor joined
10:38:46.782Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:38:46.787Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:38:46.858Z INFO syncengine_core::sync::gossip - Neighbor joined
10:38:46.858Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:38:46.863Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:38:46.911Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:38:46.920Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:38:47.129Z INFO syncengine_core::sync::contact_handler - Received contact request, saved as IncomingPending
10:38:47.905Z INFO syncengine_core::sync::contact_manager - Auto-reconnecting to saved contacts
10:38:47.905Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:47.905Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:47.905Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:38:47.905Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:38:47.906Z INFO syncengine_core::sync::gossip - Neighbor joined
10:38:47.906Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:38:47.906Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:38:47.906Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:47.906Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:47.906Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic on reconnect
10:38:47.906Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
10:38:47.910Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:38:48.008Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:48.008Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:48.008Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:38:48.008Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:38:48.008Z INFO syncengine_core::sync::gossip - Neighbor joined
10:38:48.008Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:38:48.008Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:38:48.008Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:48.008Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:48.008Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic on reconnect
10:38:48.008Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
10:38:48.014Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:38:48.115Z INFO syncengine_core::engine - Profile announcement broadcast on contact topics
10:38:48.115Z INFO syncengine_core::engine - Profile announcement broadcast complete
10:38:48.115Z INFO syncengine_core::engine - Presence announced on profile and contact topics
10:38:48.115Z INFO syncengine_core::engine - Attempting startup sync with known peers
10:38:48.115Z INFO syncengine_core::engine - Startup sync complete
10:38:48.115Z INFO syncengine_desktop::app - Startup sync complete: 2 succeeded, 0 attempted, 0 skipped (backoff), jitter=1174ms
10:38:48.115Z INFO syncengine_desktop::app - SyncEngine initialized with identity
10:38:48.116Z INFO syncengine_desktop::pages::landing - Returning user detected (1 realms), auto-navigating to Network
10:38:48.122Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:48.124Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:38:48.124Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:38:48.124Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:48.124Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:38:48.124Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:38:48.139Z INFO syncengine_core::engine - Loaded historical packet events from storage
10:38:49.136Z INFO syncengine_core::sync::contact_manager - Updated contact profile from per-peer topic
10:38:49.136Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:49.136Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:38:49.136Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:38:49.136Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:49.136Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:38:49.136Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:38:49.144Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:38:49.144Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:38:49.144Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:38:49.144Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:38:49.151Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:38:49.151Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:49.151Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:38:49.151Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:38:49.151Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:49.151Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:38:49.151Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:38:49.156Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:38:49.160Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:49.160Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:38:49.160Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:38:49.160Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:49.160Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:38:49.160Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:38:49.164Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:38:49.164Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:49.164Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:38:49.164Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:38:49.164Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:49.164Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:38:49.164Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:38:49.168Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:38:49.168Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:49.168Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:38:49.168Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:38:49.168Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:49.168Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:38:49.168Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:38:49.172Z INFO syncengine_core::sync::contact_manager - Updated contact profile from per-peer topic
10:38:49.172Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:49.172Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:38:49.172Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:38:49.172Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:49.172Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:38:49.172Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:38:49.176Z INFO syncengine_core::sync::contact_manager - Updated contact profile from per-peer topic
10:38:49.176Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:49.176Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:38:49.176Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:38:49.176Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:49.176Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:38:49.176Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:38:54.130Z INFO syncengine_core::profile::mirror - MirrorStore: STORING packet with key
10:38:54.136Z INFO syncengine_core::engine - Sent packet to contact via 1:1 topic
10:38:54.136Z INFO syncengine_core::engine - Packet also sent via mutual peer (proactive relay)
10:38:54.136Z INFO syncengine_core::engine - Proactive relay complete
10:38:54.136Z INFO syncengine_core::engine - Recording OUTGOING packet event
10:38:54.136Z INFO syncengine_desktop::app - Message watcher: sent message
```

</details>

## Instance: `peace`

Total: 126 entries (126 info, 0 warn, 0 error)

<details>
<summary>Click to expand logs</summary>

```log
10:38:45.458Z INFO syncengine_desktop - JSONL logging enabled
10:38:45.593Z INFO syncengine_desktop - Starting 'instance-peace' with data dir: "/Users/truman/Library/Application Support/syncengine-scenarios/instance-peace", screen: 1512x982, window: 504x957, total_windows: 3
10:38:45.960Z INFO syncengine_core::engine - Initializing SyncEngine
10:38:45.985Z INFO syncengine_core::blobs - Creating persistent blob manager with FsStore
10:38:46.037Z INFO syncengine_core::engine - Blob manager initialized with persistent storage
10:38:46.041Z INFO syncengine_core::engine - Creating default Private realm with sacred onboarding
10:38:46.054Z INFO syncengine_core::engine - Generating new identity
10:38:46.059Z INFO syncengine_core::engine - Deriving profile keys from identity keypair
10:38:46.072Z INFO syncengine_desktop::app - Profile name set to 'Peace'
10:38:46.072Z INFO syncengine_core::engine - Initializing contact manager
10:38:46.072Z INFO syncengine_core::engine - Initializing gossip networking
10:38:46.072Z INFO syncengine_core::engine - No endpoint secret key found, generating new one
10:38:46.077Z INFO syncengine_core::engine - Saved new endpoint secret key to storage
10:38:46.087Z INFO syncengine_core::sync::gossip - Endpoint bound
10:38:46.087Z INFO syncengine_core::sync::gossip - Gossip spawned
10:38:46.087Z INFO syncengine_core::sync::gossip - Contact protocol handler registered with shared active_topics
10:38:46.087Z INFO syncengine_core::sync::gossip - Profile protocol handler registered
10:38:46.087Z INFO syncengine_core::sync::gossip - Blob protocol handler registered for P2P image transfer
10:38:46.087Z INFO syncengine_core::sync::gossip - Router spawned
10:38:46.098Z INFO syncengine_core::sync::contact_manager - Contact auto-accept task started
10:38:46.099Z INFO syncengine_core::sync::contact_manager - Contact subscription task started
10:38:46.099Z INFO syncengine_core::engine - Contact accepted profile announcer started
10:38:46.099Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:46.099Z INFO syncengine_core::engine - Subscribed to own profile topic for broadcasting
10:38:46.099Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:46.099Z INFO syncengine_core::engine - Profile sync started on global topic
10:38:46.099Z INFO syncengine_core::engine - Profile sync listener started with P2P blob download support
10:38:46.099Z INFO syncengine_core::sync::contact_manager - Generated hybrid contact invite (v2)
10:38:46.107Z INFO syncengine_desktop::app - Bootstrap invite written to "/Users/truman/Library/Application Support/syncengine-bootstrap/peace.invite"
10:38:46.222Z INFO syncengine_core::sync::contact_handler - Received request for our own invite - will auto-accept
10:38:46.232Z INFO syncengine_core::sync::contact_handler - Received contact request, saved as IncomingPending
10:38:46.233Z INFO syncengine_core::sync::contact_manager - Auto-accepting contact request for our own invite
10:38:46.335Z INFO syncengine_core::sync::contact_manager - Accepting contact request with simplified 2-message protocol
10:38:46.338Z INFO syncengine_core::sync::contact_manager - Sent ContactAccept (simplified protocol)
10:38:46.601Z INFO syncengine_core::sync::contact_handler - Received contact request, saved as IncomingPending
10:38:46.615Z INFO syncengine_core::sync::contact_manager - Saved outgoing contact request
10:38:46.616Z INFO syncengine_core::sync::contact_manager - Sent ContactRequest
10:38:46.854Z INFO syncengine_core::sync::contact_manager - Pinned contact's profile from contact exchange
10:38:46.858Z INFO syncengine_core::sync::contact_manager - Finalized contact and saved to database (unified peer system)
10:38:46.858Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:46.858Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:46.858Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:38:46.858Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:38:46.858Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:38:46.858Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:46.858Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:46.858Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic for updates
10:38:46.858Z INFO syncengine_core::sync::contact_manager - Successfully auto-accepted contact request
10:38:46.858Z INFO syncengine_core::engine - Contact accepted, announcing profile for auto-pinning
10:38:46.858Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
10:38:46.858Z INFO syncengine_core::sync::contact_manager - ContactAccepted event received, ensuring topic subscription for receiving
10:38:46.858Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:46.858Z INFO syncengine_core::engine - Profile announced after contact accepted - peer should auto-pin
10:38:46.858Z INFO syncengine_core::sync::gossip - Neighbor joined
10:38:46.858Z INFO syncengine_core::sync::gossip - Neighbor joined
10:38:46.858Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:38:46.859Z INFO syncengine_core::sync::gossip - Neighbor joined
10:38:46.867Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:38:46.911Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:46.911Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:46.911Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:38:46.911Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:38:46.911Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:38:46.911Z INFO syncengine_core::sync::contact_manager - Successfully subscribed to contact topic for receiving messages
10:38:46.911Z INFO syncengine_core::sync::gossip - Neighbor joined
10:38:46.911Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:38:46.916Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:38:47.117Z INFO syncengine_desktop::app - Bootstrap: sent contact request to 'joy' (attempt 1)
10:38:47.122Z INFO syncengine_core::sync::contact_manager - Saved outgoing contact request
10:38:47.123Z INFO syncengine_core::sync::contact_manager - Sent ContactRequest
10:38:47.625Z INFO syncengine_desktop::app - Bootstrap: sent contact request to 'love' (attempt 1)
10:38:47.625Z INFO syncengine_desktop::app - Bootstrap: all peers connected successfully
10:38:47.625Z INFO syncengine_core::engine - Starting startup sync...
10:38:48.009Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:38:48.009Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:38:48.018Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:38:48.022Z INFO syncengine_core::sync::contact_manager - Updated contact profile from contact topic
10:38:48.124Z INFO syncengine_core::sync::contact_manager - Updated contact profile from per-peer topic
10:38:49.023Z INFO syncengine_core::sync::contact_manager - Auto-reconnecting to saved contacts
10:38:49.023Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:49.023Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:49.023Z INFO syncengine_core::sync::contact_manager - Subscribed to contact gossip topic with profile listener
10:38:49.023Z INFO syncengine_core::sync::contact_manager - Contact topic listener STARTED - attempting to receive events
10:38:49.023Z INFO syncengine_core::sync::gossip - Neighbor joined
10:38:49.023Z INFO syncengine_core::sync::contact_manager - Neighbor UP on contact topic - mesh forming
10:38:49.024Z INFO syncengine_core::sync::contact_manager - Sent mesh formation announcement to contact topic
10:38:49.024Z INFO syncengine_core::sync::gossip - Adding peer address to static discovery
10:38:49.024Z INFO syncengine_core::sync::gossip - Subscribing to topic (split)
10:38:49.024Z INFO syncengine_core::sync::contact_manager - Subscribed to contact's profile topic on reconnect
10:38:49.024Z INFO syncengine_core::sync::contact_manager - Profile topic listener started for contact
10:38:49.024Z INFO syncengine_core::sync::gossip - Neighbor joined
10:38:49.024Z INFO syncengine_core::sync::gossip - Neighbor joined
10:38:49.028Z INFO syncengine_core::profile::mirror - Retrieved packets for recipient (relay delivery)
10:38:49.131Z INFO syncengine_core::engine - Profile announcement broadcast on contact topics
10:38:49.131Z INFO syncengine_core::engine - Profile announcement broadcast complete
10:38:49.131Z INFO syncengine_core::engine - Presence announced on profile and contact topics
10:38:49.131Z INFO syncengine_core::engine - Attempting startup sync with known peers
10:38:49.131Z INFO syncengine_core::engine - Startup sync complete
10:38:49.131Z INFO syncengine_desktop::app - Startup sync complete: 1 succeeded, 0 attempted, 0 skipped (backoff), jitter=1396ms
10:38:49.131Z INFO syncengine_desktop::app - SyncEngine initialized with identity
10:38:49.131Z INFO syncengine_desktop::pages::landing - Returning user detected (1 realms), auto-navigating to Network
10:38:49.135Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:49.136Z INFO syncengine_core::engine - get_conversation: loaded 0 received packets from MirrorStore
10:38:49.136Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=0
10:38:49.141Z INFO syncengine_core::engine - Loaded historical packet events from storage
10:38:54.224Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:38:54.224Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:38:54.224Z INFO syncengine_core::sync::contact_manager - PARSED PACKET - checking DID match
10:38:54.224Z INFO syncengine_core::sync::contact_manager - RECEIVED gossip message on contact topic
10:38:54.224Z INFO syncengine_core::sync::contact_manager - PARSED PACKET - checking DID match
10:38:54.224Z INFO syncengine_core::sync::contact_manager - PARSED PACKET - checking DID match
10:38:54.228Z INFO syncengine_core::profile::mirror - MirrorStore: STORING packet with key
10:38:54.232Z INFO syncengine_core::profile::mirror - MirrorStore: STORING packet with key
10:38:54.236Z INFO syncengine_core::profile::mirror - MirrorStore: STORING packet with key
10:38:54.240Z INFO syncengine_core::sync::contact_manager - Stored packet from contact via 1:1 topic
10:38:54.240Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:54.240Z INFO syncengine_core::engine - get_conversation: loaded 1 received packets from MirrorStore
10:38:54.241Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=1
10:38:54.244Z INFO syncengine_core::sync::contact_manager - Stored packet from contact via 1:1 topic
10:38:54.244Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:54.244Z INFO syncengine_core::engine - get_conversation: loaded 1 received packets from MirrorStore
10:38:54.244Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=1
10:38:54.248Z INFO syncengine_core::sync::contact_manager - Stored packet from contact via 1:1 topic
10:38:54.248Z INFO syncengine_core::engine - get_conversation: loading packets from MirrorStore
10:38:54.248Z INFO syncengine_core::engine - get_conversation: loaded 1 received packets from MirrorStore
10:38:54.248Z INFO syncengine_core::engine - get_conversation: SUMMARY - sent=0, received=1
```

</details>

## Instance: `scenario-mesh-messaging`

Total: 48 entries (48 info, 0 warn, 0 error)

<details>
<summary>Click to expand logs</summary>

```log
10:38:44.436Z INFO syncengine_scenario - JSONL logging enabled for scenario
10:38:44.436Z INFO syncengine_scenario - Loading scenario: mesh-messaging
10:38:44.438Z INFO syncengine_scenarios::runtime - Running scenario
10:38:44.438Z INFO syncengine_scenarios::runtime - Parsed instance names from scenario
10:38:44.438Z INFO syncengine_scenarios::runtime - Starting instance launches
10:38:44.438Z INFO syncengine_scenarios::runtime - Launching instance
10:38:44.438Z INFO syncengine_scenarios::instance - Launching instance
10:38:44.439Z INFO syncengine_scenarios::runtime - Instance launch succeeded
10:38:44.941Z INFO syncengine_scenarios::runtime - Launching instance
10:38:44.941Z INFO syncengine_scenarios::instance - Launching instance
10:38:44.941Z INFO syncengine_scenarios::runtime - Instance launch succeeded
10:38:45.446Z INFO syncengine_scenarios::runtime - Launching instance
10:38:45.450Z INFO syncengine_scenarios::instance - Launching instance
10:38:45.451Z INFO syncengine_scenarios::runtime - Instance launch succeeded
10:38:45.956Z INFO syncengine_scenarios::runtime - Finished launching instances
10:38:45.956Z INFO syncengine_scenarios::runtime - Created mesh topology with auto-connect
10:38:45.956Z INFO scenario - === Direct Messaging Test (Full Mesh) ===
10:38:45.956Z INFO scenario - Topology: love <-> joy <-> peace <-> love
10:38:45.956Z INFO scenario - All nodes are connected to each other
10:38:45.956Z INFO scenario - 
10:38:53.958Z INFO scenario - Step 1: Love sends message to Peace...
10:38:53.958Z INFO syncengine_scenarios::api - Wrote message instruction to love.sendmsg
10:38:55.958Z INFO scenario - Step 2: Peace sends message to Joy...
10:38:55.959Z INFO syncengine_scenarios::api - Wrote message instruction to peace.sendmsg
10:38:57.958Z INFO scenario - Step 3: Joy sends message to Love...
10:38:57.959Z INFO syncengine_scenarios::api - Wrote message instruction to joy.sendmsg
10:38:59.959Z INFO scenario - Step 4: Testing reverse directions...
10:38:59.959Z INFO syncengine_scenarios::api - Wrote message instruction to peace.sendmsg
10:38:59.960Z INFO syncengine_scenarios::api - Wrote message instruction to joy.sendmsg
10:38:59.960Z INFO syncengine_scenarios::api - Wrote message instruction to love.sendmsg
10:39:02.958Z INFO scenario - 
10:39:02.958Z INFO scenario - === Test Complete ===
10:39:02.958Z INFO scenario - Each instance should have received 2 messages:
10:39:02.958Z INFO scenario -   - Love: from Joy and Peace
10:39:02.958Z INFO scenario -   - Joy: from Love and Peace
10:39:02.958Z INFO scenario -   - Peace: from Love and Joy
10:39:02.958Z INFO scenario - 
10:39:02.958Z INFO scenario - Check packet count in each instance's Network tab
10:39:02.958Z INFO scenario - Press Ctrl+C to stop all instances
10:39:02.958Z INFO syncengine_scenarios::runtime - Scenario running. Press Ctrl+C to stop, or quit all instances to exit.
10:39:03.460Z INFO syncengine_scenarios::instance - Instance exited
10:39:03.461Z INFO syncengine_scenarios::instance - Instance exited
10:39:03.461Z INFO syncengine_scenarios::instance - Instance exited
10:39:03.461Z INFO syncengine_scenarios::runtime - All instances have exited, shutting down scenario...
10:39:03.461Z INFO syncengine_scenario - Scenario 'mesh-messaging' completed
10:39:03.461Z INFO syncengine_scenarios::instance - Killing instance
10:39:03.461Z INFO syncengine_scenarios::instance - Killing instance
10:39:03.461Z INFO syncengine_scenarios::instance - Killing instance
```

</details>

---

*Generated from JSONL logs. Regenerate with: `syncengine-cli logs report`*
