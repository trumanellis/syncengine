# Synchronicity Engine - Run Logs

**Started:** 2026-01-20 05:51:11

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
[2m2026-01-20T05:51:33.115320Z[0m [32m INFO[0m [2msyncengine_desktop[0m[2m:[0m Starting 'love' with data dir: "/Users/truman/Library/Application Support/instance-love", screen: 1512x982, window: 756x957, total_windows: 2
[2m2026-01-20T05:51:33.125618Z[0m [32m INFO[0m [2msyncengine_desktop[0m[2m:[0m Starting 'joy' with data dir: "/Users/truman/Library/Application Support/instance-joy", screen: 1512x982, window: 756x957, total_windows: 2
[2m2026-01-20T05:51:33.576017Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing SyncEngine [3mdata_dir[0m[2m=[0m"/Users/truman/Library/Application Support/instance-love"
[2m2026-01-20T05:51:33.576895Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing SyncEngine [3mdata_dir[0m[2m=[0m"/Users/truman/Library/Application Support/instance-joy"
[2m2026-01-20T05:51:33.656435Z[0m [32m INFO[0m [2msyncengine_core::blobs[0m[2m:[0m Creating persistent blob manager with FsStore [3mpath[0m[2m=[0m"/Users/truman/Library/Application Support/instance-love/blobs"
[2m2026-01-20T05:51:33.658731Z[0m [32m INFO[0m [2msyncengine_core::blobs[0m[2m:[0m Creating persistent blob manager with FsStore [3mpath[0m[2m=[0m"/Users/truman/Library/Application Support/instance-joy/blobs"
[2m2026-01-20T05:51:33.711332Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Blob manager initialized with persistent storage [3mblob_path[0m[2m=[0m"/Users/truman/Library/Application Support/instance-love/blobs"
[2m2026-01-20T05:51:33.714918Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Blob manager initialized with persistent storage [3mblob_path[0m[2m=[0m"/Users/truman/Library/Application Support/instance-joy/blobs"
[2m2026-01-20T05:51:33.719236Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded existing identity
[2m2026-01-20T05:51:33.719244Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Starting startup sync...
[2m2026-01-20T05:51:33.719262Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing gossip networking
[2m2026-01-20T05:51:33.719290Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded persistent endpoint secret key from storage
[2m2026-01-20T05:51:33.724953Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded existing identity
[2m2026-01-20T05:51:33.724961Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Starting startup sync...
[2m2026-01-20T05:51:33.724965Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing gossip networking
[2m2026-01-20T05:51:33.724973Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded persistent endpoint secret key from storage
[2m2026-01-20T05:51:33.732888Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Endpoint bound [3mendpoint_id[0m[2m=[0me11499f8ae1a2ee9de3d014e9192a6935cc9d981049afd1541a42a9d11883c4a
[2m2026-01-20T05:51:33.733241Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Gossip spawned [3mmax_message_size[0m[2m=[0m1048576
[2m2026-01-20T05:51:33.733281Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Contact protocol handler registered
[2m2026-01-20T05:51:33.733283Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Profile protocol handler registered
[2m2026-01-20T05:51:33.733402Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Blob protocol handler registered for P2P image transfer
[2m2026-01-20T05:51:33.733420Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Router spawned
[2m2026-01-20T05:51:33.733530Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(6bda1fedaa35cf0c073de00a752d0d7336ad4bbde1f423745d34821a6f1addd4) [3mpeer_count[0m[2m=[0m0
[2m2026-01-20T05:51:33.733662Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Subscribed to own profile topic for broadcasting [3mdid[0m[2m=[0mdid:sync:zCmjrdUSiBcFSBbqKuZ2pWPkAdHuH3h8SdngeUhAMx5Fh [3mown_topic_id[0m[2m=[0mTopicId(6bda1fedaa35cf0c073de00a752d0d7336ad4bbde1f423745d34821a6f1addd4)
[2m2026-01-20T05:51:33.733667Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(fefc9e40f7d91e18f1948815ca1dbcfc6bd239bc4ecf7f7616c1dac94a05cb1a) [3mpeer_count[0m[2m=[0m0
[2m2026-01-20T05:51:33.733681Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile sync started on global topic
[2m2026-01-20T05:51:33.733761Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile sync listener started with P2P blob download support
[2m2026-01-20T05:51:33.734299Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Endpoint bound [3mendpoint_id[0m[2m=[0m63be9db4b4502b0203629dc6ca212f986fc5edb228b7fce949c8143472f856d4
[2m2026-01-20T05:51:33.734496Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Gossip spawned [3mmax_message_size[0m[2m=[0m1048576
[2m2026-01-20T05:51:33.734539Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Contact protocol handler registered
[2m2026-01-20T05:51:33.734542Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Profile protocol handler registered
[2m2026-01-20T05:51:33.734596Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Blob protocol handler registered for P2P image transfer
[2m2026-01-20T05:51:33.734615Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Router spawned
[2m2026-01-20T05:51:33.734717Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(51b45baf554f090f6ef7bdfc5badf0636d3a7ad7ef68efb0a7e7faea4b2e4ec4) [3mpeer_count[0m[2m=[0m0
[2m2026-01-20T05:51:33.734730Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Subscribed to own profile topic for broadcasting [3mdid[0m[2m=[0mdid:sync:zCWd5UFx9yUFK2de1jWqxfurj2QA5GweRovgLFj2x1JgC [3mown_topic_id[0m[2m=[0mTopicId(51b45baf554f090f6ef7bdfc5badf0636d3a7ad7ef68efb0a7e7faea4b2e4ec4)
[2m2026-01-20T05:51:33.734734Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(fefc9e40f7d91e18f1948815ca1dbcfc6bd239bc4ecf7f7616c1dac94a05cb1a) [3mpeer_count[0m[2m=[0m0
[2m2026-01-20T05:51:33.734746Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile sync started on global topic
[2m2026-01-20T05:51:33.734856Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile sync listener started with P2P blob download support
[2m2026-01-20T05:51:33.874396Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing contact manager
[2m2026-01-20T05:51:33.880030Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Contact auto-accept task started
[2m2026-01-20T05:51:33.880046Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Contact accepted profile announcer started
[2m2026-01-20T05:51:33.880136Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Auto-reconnecting to saved contacts [3mcount[0m[2m=[0m1
[2m2026-01-20T05:51:33.880157Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Adding peer address to static discovery [3mpeer[0m[2m=[0me11499f8ae1a2ee9de3d014e9192a6935cc9d981049afd1541a42a9d11883c4a [3maddrs[0m[2m=[0m6
[2m2026-01-20T05:51:33.880175Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(900c0cab0ee53eb31dc07af6fe5f70e8da1140624b5d7ec6380d2eddb2b989a4) [3mpeer_count[0m[2m=[0m1
[2m2026-01-20T05:51:33.880203Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Subscribed to contact gossip topic with profile listener [3mpeer_did[0m[2m=[0mdid:sync:zCmjrdUSiBcFSBbqKuZ2pWPkAdHuH3h8SdngeUhAMx5Fh [3mtopic_id[0m[2m=[0mTopicId(900c0cab0ee53eb31dc07af6fe5f70e8da1140624b5d7ec6380d2eddb2b989a4)
[2m2026-01-20T05:51:33.880213Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Adding peer address to static discovery [3mpeer[0m[2m=[0me11499f8ae1a2ee9de3d014e9192a6935cc9d981049afd1541a42a9d11883c4a [3maddrs[0m[2m=[0m6
[2m2026-01-20T05:51:33.880220Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(6bda1fedaa35cf0c073de00a752d0d7336ad4bbde1f423745d34821a6f1addd4) [3mpeer_count[0m[2m=[0m1
[2m2026-01-20T05:51:33.880225Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Subscribed to contact's profile topic on reconnect [3mpeer_did[0m[2m=[0mdid:sync:zCmjrdUSiBcFSBbqKuZ2pWPkAdHuH3h8SdngeUhAMx5Fh [3mpeer_profile_topic[0m[2m=[0mTopicId(6bda1fedaa35cf0c073de00a752d0d7336ad4bbde1f423745d34821a6f1addd4)
[2m2026-01-20T05:51:33.880235Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Profile topic listener started for contact [3mpeer_did[0m[2m=[0mdid:sync:zCmjrdUSiBcFSBbqKuZ2pWPkAdHuH3h8SdngeUhAMx5Fh
[2m2026-01-20T05:51:33.990473Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile announcement broadcast on contact topics [3mcontacts_updated[0m[2m=[0m1
[2m2026-01-20T05:51:33.990484Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile announcement broadcast complete
[2m2026-01-20T05:51:33.990486Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Presence announced on profile and contact topics
[2m2026-01-20T05:51:33.990508Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Attempting startup sync with known peers [3mpeer_count[0m[2m=[0m1
[2m2026-01-20T05:51:35.012748Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing contact manager
[2m2026-01-20T05:51:35.028688Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Contact auto-accept task started
[2m2026-01-20T05:51:35.028703Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Contact accepted profile announcer started
[2m2026-01-20T05:51:35.028725Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Auto-reconnecting to saved contacts [3mcount[0m[2m=[0m1
[2m2026-01-20T05:51:35.028744Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Adding peer address to static discovery [3mpeer[0m[2m=[0m63be9db4b4502b0203629dc6ca212f986fc5edb228b7fce949c8143472f856d4 [3maddrs[0m[2m=[0m6
[2m2026-01-20T05:51:35.028752Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(900c0cab0ee53eb31dc07af6fe5f70e8da1140624b5d7ec6380d2eddb2b989a4) [3mpeer_count[0m[2m=[0m1
[2m2026-01-20T05:51:35.028769Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Subscribed to contact gossip topic with profile listener [3mpeer_did[0m[2m=[0mdid:sync:zCWd5UFx9yUFK2de1jWqxfurj2QA5GweRovgLFj2x1JgC [3mtopic_id[0m[2m=[0mTopicId(900c0cab0ee53eb31dc07af6fe5f70e8da1140624b5d7ec6380d2eddb2b989a4)
[2m2026-01-20T05:51:35.028779Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Adding peer address to static discovery [3mpeer[0m[2m=[0m63be9db4b4502b0203629dc6ca212f986fc5edb228b7fce949c8143472f856d4 [3maddrs[0m[2m=[0m6
[2m2026-01-20T05:51:35.028785Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(51b45baf554f090f6ef7bdfc5badf0636d3a7ad7ef68efb0a7e7faea4b2e4ec4) [3mpeer_count[0m[2m=[0m1
[2m2026-01-20T05:51:35.028791Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Subscribed to contact's profile topic on reconnect [3mpeer_did[0m[2m=[0mdid:sync:zCWd5UFx9yUFK2de1jWqxfurj2QA5GweRovgLFj2x1JgC [3mpeer_profile_topic[0m[2m=[0mTopicId(51b45baf554f090f6ef7bdfc5badf0636d3a7ad7ef68efb0a7e7faea4b2e4ec4)
[2m2026-01-20T05:51:35.028829Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Profile topic listener started for contact [3mpeer_did[0m[2m=[0mdid:sync:zCWd5UFx9yUFK2de1jWqxfurj2QA5GweRovgLFj2x1JgC
[2m2026-01-20T05:51:35.147740Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile announcement broadcast on contact topics [3mcontacts_updated[0m[2m=[0m1
[2m2026-01-20T05:51:35.147752Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile announcement broadcast complete
[2m2026-01-20T05:51:35.147754Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Presence announced on profile and contact topics
[2m2026-01-20T05:51:35.147770Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Attempting startup sync with known peers [3mpeer_count[0m[2m=[0m1
[2m2026-01-20T05:51:35.642389Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Connected on startup [3mpeer_id[0m[2m=[0mPublicKey(63be9db4b4502b0203629dc6ca212f986fc5edb228b7fce949c8143472f856d4) [3mis_contact[0m[2m=[0mtrue [3msuccess_rate[0m[2m=[0m"120.0%"
[2m2026-01-20T05:51:35.653302Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Startup sync complete [3mattempted[0m[2m=[0m1 [3msucceeded[0m[2m=[0m1 [3mskipped[0m[2m=[0m0 [3mjitter_ms[0m[2m=[0m1277
[2m2026-01-20T05:51:35.653313Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m Startup sync complete: 1 succeeded, 1 attempted, 0 skipped (backoff), jitter=1277ms
[2m2026-01-20T05:51:35.653323Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m SyncEngine initialized with identity
[2m2026-01-20T05:51:35.653383Z[0m [32m INFO[0m [2msyncengine_desktop::pages::landing[0m[2m:[0m Returning user detected (1 realms), auto-navigating to Field
[2m2026-01-20T05:51:35.657558Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Opening realm [3mrealm_id[0m[2m=[0mrealm_LSbCatvKr41
[2m2026-01-20T05:51:35.658480Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Loaded 5 tasks for realm Private
[2m2026-01-20T05:51:35.658484Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Total realms loaded: 1, total task entries: 1
[2m2026-01-20T05:51:35.658523Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Setting signals - realms: 1, tasks_by_realm entries: 1
[2m2026-01-20T05:51:35.658528Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Signals set - generation now: 1, data_loaded: true
[2m2026-01-20T05:51:35.658606Z[0m [32m INFO[0m [2msyncengine_desktop::components::unified_field[0m[2m:[0m UnifiedFieldView rendering - realms: 1, tasks_by_realm entries: 1, generation: 1
[2m2026-01-20T05:51:35.658617Z[0m [32m INFO[0m [2msyncengine_desktop::components::unified_field[0m[2m:[0m Rendering realm Private with 5 tasks
[2m2026-01-20T05:51:35.737467Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Neighbor joined [3mtopic[0m[2m=[0mTopicId(900c0cab0ee53eb31dc07af6fe5f70e8da1140624b5d7ec6380d2eddb2b989a4) [3mpeer[0m[2m=[0mPublicKey(e11499f8ae1a2ee9de3d014e9192a6935cc9d981049afd1541a42a9d11883c4a)
[2m2026-01-20T05:51:35.737622Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Neighbor joined [3mtopic[0m[2m=[0mTopicId(900c0cab0ee53eb31dc07af6fe5f70e8da1140624b5d7ec6380d2eddb2b989a4) [3mpeer[0m[2m=[0mPublicKey(63be9db4b4502b0203629dc6ca212f986fc5edb228b7fce949c8143472f856d4)
[2m2026-01-20T05:51:35.737630Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Neighbor joined [3mtopic[0m[2m=[0mTopicId(51b45baf554f090f6ef7bdfc5badf0636d3a7ad7ef68efb0a7e7faea4b2e4ec4) [3mpeer[0m[2m=[0mPublicKey(63be9db4b4502b0203629dc6ca212f986fc5edb228b7fce949c8143472f856d4)
[2m2026-01-20T05:51:36.991484Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Connected on startup [3mpeer_id[0m[2m=[0mPublicKey(e11499f8ae1a2ee9de3d014e9192a6935cc9d981049afd1541a42a9d11883c4a) [3mis_contact[0m[2m=[0mtrue [3msuccess_rate[0m[2m=[0m"120.0%"
[2m2026-01-20T05:51:36.991748Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Neighbor left [3mtopic[0m[2m=[0mTopicId(51b45baf554f090f6ef7bdfc5badf0636d3a7ad7ef68efb0a7e7faea4b2e4ec4) [3mpeer[0m[2m=[0mPublicKey(63be9db4b4502b0203629dc6ca212f986fc5edb228b7fce949c8143472f856d4)
[2m2026-01-20T05:51:36.991750Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Neighbor left [3mtopic[0m[2m=[0mTopicId(900c0cab0ee53eb31dc07af6fe5f70e8da1140624b5d7ec6380d2eddb2b989a4) [3mpeer[0m[2m=[0mPublicKey(63be9db4b4502b0203629dc6ca212f986fc5edb228b7fce949c8143472f856d4)
[2m2026-01-20T05:51:37.004572Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Startup sync complete [3mattempted[0m[2m=[0m1 [3msucceeded[0m[2m=[0m1 [3mskipped[0m[2m=[0m0 [3mjitter_ms[0m[2m=[0m137
[2m2026-01-20T05:51:37.004579Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m Startup sync complete: 1 succeeded, 1 attempted, 0 skipped (backoff), jitter=137ms
[2m2026-01-20T05:51:37.004589Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m SyncEngine initialized with identity
[2m2026-01-20T05:51:37.004625Z[0m [32m INFO[0m [2msyncengine_desktop::pages::landing[0m[2m:[0m Returning user detected (1 realms), auto-navigating to Field
[2m2026-01-20T05:51:37.008379Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Opening realm [3mrealm_id[0m[2m=[0mrealm_6Wd4swL6Ba1
[2m2026-01-20T05:51:37.008658Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Loaded 5 tasks for realm Private
[2m2026-01-20T05:51:37.008662Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Total realms loaded: 1, total task entries: 1
[2m2026-01-20T05:51:37.008688Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Setting signals - realms: 1, tasks_by_realm entries: 1
[2m2026-01-20T05:51:37.008692Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Signals set - generation now: 1, data_loaded: true
[2m2026-01-20T05:51:37.008728Z[0m [32m INFO[0m [2msyncengine_desktop::components::unified_field[0m[2m:[0m UnifiedFieldView rendering - realms: 1, tasks_by_realm entries: 1, generation: 1
[2m2026-01-20T05:51:37.008733Z[0m [32m INFO[0m [2msyncengine_desktop::components::unified_field[0m[2m:[0m Rendering realm Private with 5 tasks
[2m2026-01-20T05:51:52.961815Z[0m [31mERROR[0m [2msyncengine_desktop::pages::network[0m[2m:[0m Failed to send message [3merror[0m[2m=[0mIdentity error: Profile keys not initialized. Call init_profile_keys() first. [3mto[0m[2m=[0mdid:sync:zCWd5UFx9yUFK2de1jWqxfurj2QA5GweRovgLFj2x1JgC
```

</details>

---

## Session Summary

- **Ended:** 2026-01-20 05:52:06
- **Duration:** ~0 minutes

### Log Statistics

| Level | Count |
|-------|-------|
| INFO  | 98 |
| WARN  | 0
0 |
| ERROR | 1 |
