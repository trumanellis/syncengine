# Synchronicity Engine - Run Logs

**Started:** 2026-01-19 19:05:07

## Session Info

- **Instances:** `love joy`
- **Log Level:** INFO

---

## Instance: `love`

<details>
<summary>Click to expand logs</summary>

```log
## Instance: `joy`

<details>
<summary>Click to expand logs</summary>

```log
[2m2026-01-19T19:05:20.602039Z[0m [32m INFO[0m [2msyncengine_desktop[0m[2m:[0m Starting 'joy' with data dir: "/Users/truman/Library/Application Support/instance-joy", screen: 1512x982, window: 756x957, total_windows: 2
[2m2026-01-19T19:05:20.612068Z[0m [32m INFO[0m [2msyncengine_desktop[0m[2m:[0m Starting 'love' with data dir: "/Users/truman/Library/Application Support/instance-love", screen: 1512x982, window: 756x957, total_windows: 2
[2m2026-01-19T19:05:21.077946Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing SyncEngine [3mdata_dir[0m[2m=[0m"/Users/truman/Library/Application Support/instance-joy"
[2m2026-01-19T19:05:21.078881Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing SyncEngine [3mdata_dir[0m[2m=[0m"/Users/truman/Library/Application Support/instance-love"
[2m2026-01-19T19:05:21.153009Z[0m [32m INFO[0m [2msyncengine_core::blobs[0m[2m:[0m Creating persistent blob manager with FsStore [3mpath[0m[2m=[0m"/Users/truman/Library/Application Support/instance-joy/blobs"
[2m2026-01-19T19:05:21.155927Z[0m [32m INFO[0m [2msyncengine_core::blobs[0m[2m:[0m Creating persistent blob manager with FsStore [3mpath[0m[2m=[0m"/Users/truman/Library/Application Support/instance-love/blobs"
[2m2026-01-19T19:05:21.203848Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Blob manager initialized with persistent storage [3mblob_path[0m[2m=[0m"/Users/truman/Library/Application Support/instance-joy/blobs"
[2m2026-01-19T19:05:21.208367Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Blob manager initialized with persistent storage [3mblob_path[0m[2m=[0m"/Users/truman/Library/Application Support/instance-love/blobs"
[2m2026-01-19T19:05:21.212310Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded existing identity
[2m2026-01-19T19:05:21.212316Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Starting startup sync...
[2m2026-01-19T19:05:21.212346Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing gossip networking
[2m2026-01-19T19:05:21.212368Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded persistent endpoint secret key from storage
[2m2026-01-19T19:05:21.217032Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded existing identity
[2m2026-01-19T19:05:21.217040Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Starting startup sync...
[2m2026-01-19T19:05:21.217042Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing gossip networking
[2m2026-01-19T19:05:21.217050Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded persistent endpoint secret key from storage
[2m2026-01-19T19:05:21.232782Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Endpoint bound [3mendpoint_id[0m[2m=[0m63be9db4b4502b0203629dc6ca212f986fc5edb228b7fce949c8143472f856d4
[2m2026-01-19T19:05:21.232835Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Endpoint bound [3mendpoint_id[0m[2m=[0me11499f8ae1a2ee9de3d014e9192a6935cc9d981049afd1541a42a9d11883c4a
[2m2026-01-19T19:05:21.233082Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Gossip spawned [3mmax_message_size[0m[2m=[0m1048576
[2m2026-01-19T19:05:21.233159Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Gossip spawned [3mmax_message_size[0m[2m=[0m1048576
[2m2026-01-19T19:05:21.233225Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Contact protocol handler registered
[2m2026-01-19T19:05:21.233232Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Profile protocol handler registered
[2m2026-01-19T19:05:21.233208Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Contact protocol handler registered
[2m2026-01-19T19:05:21.233214Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Profile protocol handler registered
[2m2026-01-19T19:05:21.233354Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Blob protocol handler registered for P2P image transfer
[2m2026-01-19T19:05:21.233350Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Blob protocol handler registered for P2P image transfer
[2m2026-01-19T19:05:21.233404Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Router spawned
[2m2026-01-19T19:05:21.233407Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Router spawned
[2m2026-01-19T19:05:21.233531Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(6bda1fedaa35cf0c073de00a752d0d7336ad4bbde1f423745d34821a6f1addd4) [3mpeer_count[0m[2m=[0m0
[2m2026-01-19T19:05:21.233516Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(51b45baf554f090f6ef7bdfc5badf0636d3a7ad7ef68efb0a7e7faea4b2e4ec4) [3mpeer_count[0m[2m=[0m0
[2m2026-01-19T19:05:21.233733Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Subscribed to own profile topic for broadcasting [3mdid[0m[2m=[0mdid:sync:zCmjrdUSiBcFSBbqKuZ2pWPkAdHuH3h8SdngeUhAMx5Fh [3mown_topic_id[0m[2m=[0mTopicId(6bda1fedaa35cf0c073de00a752d0d7336ad4bbde1f423745d34821a6f1addd4)
[2m2026-01-19T19:05:21.233741Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(fefc9e40f7d91e18f1948815ca1dbcfc6bd239bc4ecf7f7616c1dac94a05cb1a) [3mpeer_count[0m[2m=[0m0
[2m2026-01-19T19:05:21.233758Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile sync started on global topic
[2m2026-01-19T19:05:21.233891Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Subscribed to own profile topic for broadcasting [3mdid[0m[2m=[0mdid:sync:zCWd5UFx9yUFK2de1jWqxfurj2QA5GweRovgLFj2x1JgC [3mown_topic_id[0m[2m=[0mTopicId(51b45baf554f090f6ef7bdfc5badf0636d3a7ad7ef68efb0a7e7faea4b2e4ec4)
[2m2026-01-19T19:05:21.233898Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(fefc9e40f7d91e18f1948815ca1dbcfc6bd239bc4ecf7f7616c1dac94a05cb1a) [3mpeer_count[0m[2m=[0m0
[2m2026-01-19T19:05:21.233909Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile sync started on global topic
[2m2026-01-19T19:05:21.233936Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile sync listener started with P2P blob download support
[2m2026-01-19T19:05:21.234018Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile sync listener started with P2P blob download support
[2m2026-01-19T19:05:21.249830Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing contact manager
[2m2026-01-19T19:05:21.254963Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Contact auto-accept task started
[2m2026-01-19T19:05:21.254981Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Contact accepted profile announcer started
[2m2026-01-19T19:05:21.255049Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Auto-reconnecting to saved contacts [3mcount[0m[2m=[0m1
[2m2026-01-19T19:05:21.255068Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Adding peer address to static discovery [3mpeer[0m[2m=[0me11499f8ae1a2ee9de3d014e9192a6935cc9d981049afd1541a42a9d11883c4a [3maddrs[0m[2m=[0m6
[2m2026-01-19T19:05:21.255087Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(900c0cab0ee53eb31dc07af6fe5f70e8da1140624b5d7ec6380d2eddb2b989a4) [3mpeer_count[0m[2m=[0m1
[2m2026-01-19T19:05:21.255102Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Subscribed to contact gossip topic with profile listener [3mpeer_did[0m[2m=[0mdid:sync:zCmjrdUSiBcFSBbqKuZ2pWPkAdHuH3h8SdngeUhAMx5Fh [3mtopic_id[0m[2m=[0mTopicId(900c0cab0ee53eb31dc07af6fe5f70e8da1140624b5d7ec6380d2eddb2b989a4)
[2m2026-01-19T19:05:21.255111Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Adding peer address to static discovery [3mpeer[0m[2m=[0me11499f8ae1a2ee9de3d014e9192a6935cc9d981049afd1541a42a9d11883c4a [3maddrs[0m[2m=[0m6
[2m2026-01-19T19:05:21.255117Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(6bda1fedaa35cf0c073de00a752d0d7336ad4bbde1f423745d34821a6f1addd4) [3mpeer_count[0m[2m=[0m1
[2m2026-01-19T19:05:21.255122Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Subscribed to contact's profile topic on reconnect [3mpeer_did[0m[2m=[0mdid:sync:zCmjrdUSiBcFSBbqKuZ2pWPkAdHuH3h8SdngeUhAMx5Fh [3mpeer_profile_topic[0m[2m=[0mTopicId(6bda1fedaa35cf0c073de00a752d0d7336ad4bbde1f423745d34821a6f1addd4)
[2m2026-01-19T19:05:21.255146Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Profile topic listener started for contact [3mpeer_did[0m[2m=[0mdid:sync:zCmjrdUSiBcFSBbqKuZ2pWPkAdHuH3h8SdngeUhAMx5Fh
[2m2026-01-19T19:05:21.363808Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile announcement broadcast on contact topics [3mcontacts_updated[0m[2m=[0m1
[2m2026-01-19T19:05:21.363829Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile announcement broadcast complete
[2m2026-01-19T19:05:21.363855Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Presence announced on profile and contact topics
[2m2026-01-19T19:05:21.363913Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Attempting startup sync with known peers [3mpeer_count[0m[2m=[0m1
[2m2026-01-19T19:05:21.865614Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing contact manager
[2m2026-01-19T19:05:21.871438Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Contact auto-accept task started
[2m2026-01-19T19:05:21.871485Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Contact accepted profile announcer started
[2m2026-01-19T19:05:21.871567Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Auto-reconnecting to saved contacts [3mcount[0m[2m=[0m1
[2m2026-01-19T19:05:21.871637Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Adding peer address to static discovery [3mpeer[0m[2m=[0m63be9db4b4502b0203629dc6ca212f986fc5edb228b7fce949c8143472f856d4 [3maddrs[0m[2m=[0m6
[2m2026-01-19T19:05:21.871663Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(900c0cab0ee53eb31dc07af6fe5f70e8da1140624b5d7ec6380d2eddb2b989a4) [3mpeer_count[0m[2m=[0m1
[2m2026-01-19T19:05:21.871710Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Subscribed to contact gossip topic with profile listener [3mpeer_did[0m[2m=[0mdid:sync:zCWd5UFx9yUFK2de1jWqxfurj2QA5GweRovgLFj2x1JgC [3mtopic_id[0m[2m=[0mTopicId(900c0cab0ee53eb31dc07af6fe5f70e8da1140624b5d7ec6380d2eddb2b989a4)
[2m2026-01-19T19:05:21.871741Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Adding peer address to static discovery [3mpeer[0m[2m=[0m63be9db4b4502b0203629dc6ca212f986fc5edb228b7fce949c8143472f856d4 [3maddrs[0m[2m=[0m6
[2m2026-01-19T19:05:21.871774Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(51b45baf554f090f6ef7bdfc5badf0636d3a7ad7ef68efb0a7e7faea4b2e4ec4) [3mpeer_count[0m[2m=[0m1
[2m2026-01-19T19:05:21.871784Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Subscribed to contact's profile topic on reconnect [3mpeer_did[0m[2m=[0mdid:sync:zCWd5UFx9yUFK2de1jWqxfurj2QA5GweRovgLFj2x1JgC [3mpeer_profile_topic[0m[2m=[0mTopicId(51b45baf554f090f6ef7bdfc5badf0636d3a7ad7ef68efb0a7e7faea4b2e4ec4)
[2m2026-01-19T19:05:21.871964Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Profile topic listener started for contact [3mpeer_did[0m[2m=[0mdid:sync:zCWd5UFx9yUFK2de1jWqxfurj2QA5GweRovgLFj2x1JgC
[2m2026-01-19T19:05:21.979184Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile announcement broadcast on contact topics [3mcontacts_updated[0m[2m=[0m1
[2m2026-01-19T19:05:21.979226Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile announcement broadcast complete
[2m2026-01-19T19:05:21.979238Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Presence announced on profile and contact topics
[2m2026-01-19T19:05:21.979362Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Attempting startup sync with known peers [3mpeer_count[0m[2m=[0m1
[2m2026-01-19T19:05:22.366009Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Neighbor joined [3mtopic[0m[2m=[0mTopicId(900c0cab0ee53eb31dc07af6fe5f70e8da1140624b5d7ec6380d2eddb2b989a4) [3mpeer[0m[2m=[0mPublicKey(63be9db4b4502b0203629dc6ca212f986fc5edb228b7fce949c8143472f856d4)
[2m2026-01-19T19:05:22.366008Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Neighbor joined [3mtopic[0m[2m=[0mTopicId(900c0cab0ee53eb31dc07af6fe5f70e8da1140624b5d7ec6380d2eddb2b989a4) [3mpeer[0m[2m=[0mPublicKey(e11499f8ae1a2ee9de3d014e9192a6935cc9d981049afd1541a42a9d11883c4a)
[2m2026-01-19T19:05:22.366421Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Neighbor joined [3mtopic[0m[2m=[0mTopicId(6bda1fedaa35cf0c073de00a752d0d7336ad4bbde1f423745d34821a6f1addd4) [3mpeer[0m[2m=[0mPublicKey(e11499f8ae1a2ee9de3d014e9192a6935cc9d981049afd1541a42a9d11883c4a)
[2m2026-01-19T19:05:22.366440Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Neighbor joined [3mtopic[0m[2m=[0mTopicId(51b45baf554f090f6ef7bdfc5badf0636d3a7ad7ef68efb0a7e7faea4b2e4ec4) [3mpeer[0m[2m=[0mPublicKey(63be9db4b4502b0203629dc6ca212f986fc5edb228b7fce949c8143472f856d4)
[2m2026-01-19T19:05:22.408940Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Connected on startup [3mpeer_id[0m[2m=[0mPublicKey(63be9db4b4502b0203629dc6ca212f986fc5edb228b7fce949c8143472f856d4) [3mis_contact[0m[2m=[0mtrue [3msuccess_rate[0m[2m=[0m"150.0%"
[2m2026-01-19T19:05:22.409426Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Neighbor left [3mtopic[0m[2m=[0mTopicId(900c0cab0ee53eb31dc07af6fe5f70e8da1140624b5d7ec6380d2eddb2b989a4) [3mpeer[0m[2m=[0mPublicKey(e11499f8ae1a2ee9de3d014e9192a6935cc9d981049afd1541a42a9d11883c4a)
[2m2026-01-19T19:05:22.409448Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Neighbor left [3mtopic[0m[2m=[0mTopicId(6bda1fedaa35cf0c073de00a752d0d7336ad4bbde1f423745d34821a6f1addd4) [3mpeer[0m[2m=[0mPublicKey(e11499f8ae1a2ee9de3d014e9192a6935cc9d981049afd1541a42a9d11883c4a)
[2m2026-01-19T19:05:22.414958Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Startup sync complete [3mattempted[0m[2m=[0m1 [3msucceeded[0m[2m=[0m1 [3mskipped[0m[2m=[0m0 [3mjitter_ms[0m[2m=[0m631
[2m2026-01-19T19:05:22.414987Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m Startup sync complete: 1 succeeded, 1 attempted, 0 skipped (backoff), jitter=631ms
[2m2026-01-19T19:05:22.415124Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m SyncEngine initialized with identity
[2m2026-01-19T19:05:22.415268Z[0m [32m INFO[0m [2msyncengine_desktop::pages::landing[0m[2m:[0m Returning user detected (1 realms), auto-navigating to Field
[2m2026-01-19T19:05:22.431318Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Opening realm [3mrealm_id[0m[2m=[0mrealm_LSbCatvKr41
[2m2026-01-19T19:05:22.433358Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Loaded 5 tasks for realm Private
[2m2026-01-19T19:05:22.433366Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Total realms loaded: 1, total task entries: 1
[2m2026-01-19T19:05:22.433427Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Setting signals - realms: 1, tasks_by_realm entries: 1
[2m2026-01-19T19:05:22.433433Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Signals set - generation now: 1, data_loaded: true
[2m2026-01-19T19:05:22.433503Z[0m [32m INFO[0m [2msyncengine_desktop::components::unified_field[0m[2m:[0m UnifiedFieldView rendering - realms: 1, tasks_by_realm entries: 1, generation: 1
[2m2026-01-19T19:05:22.433511Z[0m [32m INFO[0m [2msyncengine_desktop::components::unified_field[0m[2m:[0m Rendering realm Private with 5 tasks
[2m2026-01-19T19:05:22.461359Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Connected on startup [3mpeer_id[0m[2m=[0mPublicKey(e11499f8ae1a2ee9de3d014e9192a6935cc9d981049afd1541a42a9d11883c4a) [3mis_contact[0m[2m=[0mtrue [3msuccess_rate[0m[2m=[0m"150.0%"
[2m2026-01-19T19:05:22.461676Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Neighbor left [3mtopic[0m[2m=[0mTopicId(51b45baf554f090f6ef7bdfc5badf0636d3a7ad7ef68efb0a7e7faea4b2e4ec4) [3mpeer[0m[2m=[0mPublicKey(63be9db4b4502b0203629dc6ca212f986fc5edb228b7fce949c8143472f856d4)
[2m2026-01-19T19:05:22.461677Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Neighbor left [3mtopic[0m[2m=[0mTopicId(900c0cab0ee53eb31dc07af6fe5f70e8da1140624b5d7ec6380d2eddb2b989a4) [3mpeer[0m[2m=[0mPublicKey(63be9db4b4502b0203629dc6ca212f986fc5edb228b7fce949c8143472f856d4)
[2m2026-01-19T19:05:22.466019Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Startup sync complete [3mattempted[0m[2m=[0m1 [3msucceeded[0m[2m=[0m1 [3mskipped[0m[2m=[0m0 [3mjitter_ms[0m[2m=[0m14
[2m2026-01-19T19:05:22.466026Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m Startup sync complete: 1 succeeded, 1 attempted, 0 skipped (backoff), jitter=14ms
[2m2026-01-19T19:05:22.466041Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m SyncEngine initialized with identity
[2m2026-01-19T19:05:22.466064Z[0m [32m INFO[0m [2msyncengine_desktop::pages::landing[0m[2m:[0m Returning user detected (1 realms), auto-navigating to Field
[2m2026-01-19T19:05:22.469793Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Opening realm [3mrealm_id[0m[2m=[0mrealm_6Wd4swL6Ba1
[2m2026-01-19T19:05:22.470087Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Loaded 5 tasks for realm Private
[2m2026-01-19T19:05:22.470093Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Total realms loaded: 1, total task entries: 1
[2m2026-01-19T19:05:22.470110Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Setting signals - realms: 1, tasks_by_realm entries: 1
[2m2026-01-19T19:05:22.470114Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Signals set - generation now: 1, data_loaded: true
[2m2026-01-19T19:05:22.470152Z[0m [32m INFO[0m [2msyncengine_desktop::components::unified_field[0m[2m:[0m UnifiedFieldView rendering - realms: 1, tasks_by_realm entries: 1, generation: 1
[2m2026-01-19T19:05:22.470162Z[0m [32m INFO[0m [2msyncengine_desktop::components::unified_field[0m[2m:[0m Rendering realm Private with 5 tasks
