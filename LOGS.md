# Synchronicity Engine - Run Logs

**Started:** 2026-01-21 14:13:48

## Session Info

- **Instances:** `@offline-relay`
- **Log Level:** INFO

---

## Instance: `@offline-relay`

<details>
<summary>Click to expand logs</summary>

```log
[2m2026-01-21T14:13:50.211521Z[0m [32m INFO[0m [2msyncengine_desktop[0m[2m:[0m Starting '@offline-relay' with data dir: "/Users/truman/Library/Application Support/instance-@offline-relay", screen: 1512x982, window: 1512x957, total_windows: 1
[2m2026-01-21T14:13:50.567356Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing SyncEngine [3mdata_dir[0m[2m=[0m"/Users/truman/Library/Application Support/instance-@offline-relay"
[2m2026-01-21T14:13:50.605523Z[0m [32m INFO[0m [2msyncengine_core::blobs[0m[2m:[0m Creating persistent blob manager with FsStore [3mpath[0m[2m=[0m"/Users/truman/Library/Application Support/instance-@offline-relay/blobs"
[2m2026-01-21T14:13:50.639032Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Blob manager initialized with persistent storage [3mblob_path[0m[2m=[0m"/Users/truman/Library/Application Support/instance-@offline-relay/blobs"
[2m2026-01-21T14:13:50.648822Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded existing identity
[2m2026-01-21T14:13:50.648972Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded existing profile keys (DID matches identity)
[2m2026-01-21T14:13:50.649000Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Starting startup sync...
[2m2026-01-21T14:13:50.649030Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing gossip networking
[2m2026-01-21T14:13:50.649036Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded persistent endpoint secret key from storage
[2m2026-01-21T14:13:50.661350Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Endpoint bound [3mendpoint_id[0m[2m=[0mf66aec28abb18d932ca0d777985dd96811f48fec4713714c1a9ad197f2dbb278
[2m2026-01-21T14:13:50.661675Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Gossip spawned [3mmax_message_size[0m[2m=[0m1048576
[2m2026-01-21T14:13:50.661752Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Contact protocol handler registered with shared active_topics
[2m2026-01-21T14:13:50.661760Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Profile protocol handler registered
[2m2026-01-21T14:13:50.661825Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Blob protocol handler registered for P2P image transfer
[2m2026-01-21T14:13:50.662038Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Router spawned
[2m2026-01-21T14:13:50.662175Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(d7cacaf51636bba7ce3e5ab05950fd101eda449187b27c52e90d4ec7eeef38fe) [3mpeer_count[0m[2m=[0m0
[2m2026-01-21T14:13:50.662313Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Subscribed to own profile topic for broadcasting [3mdid[0m[2m=[0mdid:sync:z9nkNf4vKk3KsvwsghBdDYapTQ9qj8X82jNiwzpSzUMuC [3mown_topic_id[0m[2m=[0mTopicId(d7cacaf51636bba7ce3e5ab05950fd101eda449187b27c52e90d4ec7eeef38fe)
[2m2026-01-21T14:13:50.662319Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(fefc9e40f7d91e18f1948815ca1dbcfc6bd239bc4ecf7f7616c1dac94a05cb1a) [3mpeer_count[0m[2m=[0m0
[2m2026-01-21T14:13:50.662329Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile sync started on global topic
[2m2026-01-21T14:13:50.662536Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile sync listener started with P2P blob download support
[2m2026-01-21T14:13:52.111841Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing contact manager
[2m2026-01-21T14:13:52.118853Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Contact auto-accept task started
[2m2026-01-21T14:13:52.118888Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Contact subscription task started
[2m2026-01-21T14:13:52.118902Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Contact accepted profile announcer started
[2m2026-01-21T14:13:52.118923Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Auto-reconnecting to saved contacts [3mcount[0m[2m=[0m0
[2m2026-01-21T14:13:52.123661Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile announcement broadcast on contact topics [3mcontacts_updated[0m[2m=[0m0
[2m2026-01-21T14:13:52.123685Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile announcement broadcast complete
[2m2026-01-21T14:13:52.123691Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Presence announced on profile and contact topics
[2m2026-01-21T14:13:52.123709Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m Startup sync complete: 0 succeeded, 0 attempted, 0 skipped (backoff), jitter=1447ms
[2m2026-01-21T14:13:52.123765Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m SyncEngine initialized with identity
[2m2026-01-21T14:13:52.123881Z[0m [32m INFO[0m [2msyncengine_desktop::pages::landing[0m[2m:[0m Returning user detected (1 realms), auto-navigating to Field
[2m2026-01-21T14:13:52.131117Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Opening realm [3mrealm_id[0m[2m=[0mrealm_bxh2Ym3poQj
[2m2026-01-21T14:13:52.133170Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Loaded 5 tasks for realm Private
[2m2026-01-21T14:13:52.133178Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Total realms loaded: 1, total task entries: 1
[2m2026-01-21T14:13:52.133220Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Setting signals - realms: 1, tasks_by_realm entries: 1
[2m2026-01-21T14:13:52.133225Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Signals set - generation now: 1, data_loaded: true
[2m2026-01-21T14:13:52.133342Z[0m [32m INFO[0m [2msyncengine_desktop::components::unified_field[0m[2m:[0m UnifiedFieldView rendering - realms: 1, tasks_by_realm entries: 1, generation: 1
[2m2026-01-21T14:13:52.133352Z[0m [32m INFO[0m [2msyncengine_desktop::components::unified_field[0m[2m:[0m Rendering realm Private with 5 tasks
[2m2026-01-21T14:13:52.172037Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded historical packet events from storage [3mloaded_count[0m[2m=[0m0 [3mcontact_count[0m[2m=[0m0
```

</details>

---

## Session Summary

- **Ended:** 2026-01-21 14:14:19
- **Duration:** ~0 minutes

### Log Statistics

| Level | Count |
|-------|-------|
| INFO  | 40 |
| WARN  | 0
0 |
| ERROR | 0
0 |
