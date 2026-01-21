# Synchronicity Engine - Run Logs

**Started:** 2026-01-21 14:09:09

## Session Info

- **Instances:** `love`
- **Log Level:** INFO

---

## Instance: `love`

<details>
<summary>Click to expand logs</summary>

```log
[2m2026-01-21T14:09:28.418184Z[0m [32m INFO[0m [2msyncengine_desktop[0m[2m:[0m Starting 'love' with data dir: "/Users/truman/Library/Application Support/instance-love", screen: 1512x982, window: 1512x957, total_windows: 1
[2m2026-01-21T14:09:28.768151Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing SyncEngine [3mdata_dir[0m[2m=[0m"/Users/truman/Library/Application Support/instance-love"
[2m2026-01-21T14:09:28.819678Z[0m [32m INFO[0m [2msyncengine_core::blobs[0m[2m:[0m Creating persistent blob manager with FsStore [3mpath[0m[2m=[0m"/Users/truman/Library/Application Support/instance-love/blobs"
[2m2026-01-21T14:09:28.845983Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Blob manager initialized with persistent storage [3mblob_path[0m[2m=[0m"/Users/truman/Library/Application Support/instance-love/blobs"
[2m2026-01-21T14:09:28.851024Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded existing identity
[2m2026-01-21T14:09:28.851180Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded existing profile keys (DID matches identity)
[2m2026-01-21T14:09:28.851280Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded existing profile log from MirrorStore [3mpacket_count[0m[2m=[0m1 [3mhead_seq[0m[2m=[0mSome(0)
[2m2026-01-21T14:09:28.851297Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Starting startup sync...
[2m2026-01-21T14:09:28.851322Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing gossip networking
[2m2026-01-21T14:09:28.851326Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded persistent endpoint secret key from storage
[2m2026-01-21T14:09:28.862949Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Endpoint bound [3mendpoint_id[0m[2m=[0m5083f0b5b1309974dca1facf6d3d50c930b7f864d84e20865e7fd2d5fcc4587c
[2m2026-01-21T14:09:28.863360Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Gossip spawned [3mmax_message_size[0m[2m=[0m1048576
[2m2026-01-21T14:09:28.863365Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Contact protocol handler registered with shared active_topics
[2m2026-01-21T14:09:28.863367Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Profile protocol handler registered
[2m2026-01-21T14:09:28.863386Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Blob protocol handler registered for P2P image transfer
[2m2026-01-21T14:09:28.863450Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Router spawned
[2m2026-01-21T14:09:28.863503Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(db58ccf0bf7d1c4d208a799d38d40dbd9132224f53354eedb849a0e66005b4c0) [3mpeer_count[0m[2m=[0m0
[2m2026-01-21T14:09:28.863823Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Subscribed to own profile topic for broadcasting [3mdid[0m[2m=[0mdid:sync:z7r87Z6JF87gYZfFW2FVPgpskzQqnf32oTHtLmdoqYML3 [3mown_topic_id[0m[2m=[0mTopicId(db58ccf0bf7d1c4d208a799d38d40dbd9132224f53354eedb849a0e66005b4c0)
[2m2026-01-21T14:09:28.863830Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(fefc9e40f7d91e18f1948815ca1dbcfc6bd239bc4ecf7f7616c1dac94a05cb1a) [3mpeer_count[0m[2m=[0m0
[2m2026-01-21T14:09:28.863839Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile sync started on global topic
[2m2026-01-21T14:09:28.863950Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile sync listener started with P2P blob download support
[2m2026-01-21T14:09:29.815998Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing contact manager
[2m2026-01-21T14:09:29.823491Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Contact auto-accept task started
[2m2026-01-21T14:09:29.823526Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Contact subscription task started
[2m2026-01-21T14:09:29.823556Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Contact accepted profile announcer started
[2m2026-01-21T14:09:29.823688Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Auto-reconnecting to saved contacts [3mcount[0m[2m=[0m2
[2m2026-01-21T14:09:29.823773Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Adding peer address to static discovery [3mpeer[0m[2m=[0m469d37b868b512d088786a0106fc313a7e04e60d0580c0bbda7dcae3d859654f [3maddrs[0m[2m=[0m3
[2m2026-01-21T14:09:29.823792Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(cca885c940dba5004821be176c768ff279880d3cc6e062d8e6eb5e4b7f12b43b) [3mpeer_count[0m[2m=[0m1
[2m2026-01-21T14:09:29.823861Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Subscribed to contact gossip topic with profile listener [3mpeer_did[0m[2m=[0mdid:sync:z5CKkM9ErHpCxAG8ecUWUBtGfjmqbDFMpCt48Fh36oFEg [3mtopic_id[0m[2m=[0mTopicId(cca885c940dba5004821be176c768ff279880d3cc6e062d8e6eb5e4b7f12b43b) [3malready_had_sender[0m[2m=[0mfalse
[2m2026-01-21T14:09:29.823881Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Adding peer address to static discovery [3mpeer[0m[2m=[0m469d37b868b512d088786a0106fc313a7e04e60d0580c0bbda7dcae3d859654f [3maddrs[0m[2m=[0m3
[2m2026-01-21T14:09:29.823896Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(88eaffe2b70adc79c946b2788e5302db91d7bda6d52e3504b7225cb2a630a634) [3mpeer_count[0m[2m=[0m1
[2m2026-01-21T14:09:29.823995Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Subscribed to contact's profile topic on reconnect [3mpeer_did[0m[2m=[0mdid:sync:z5CKkM9ErHpCxAG8ecUWUBtGfjmqbDFMpCt48Fh36oFEg [3mpeer_profile_topic[0m[2m=[0mTopicId(88eaffe2b70adc79c946b2788e5302db91d7bda6d52e3504b7225cb2a630a634)
[2m2026-01-21T14:09:29.824331Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Contact topic listener STARTED - attempting to receive events [3mpeer_did[0m[2m=[0mdid:sync:z5CKkM9ErHpCxAG8ecUWUBtGfjmqbDFMpCt48Fh36oFEg
[2m2026-01-21T14:09:29.824374Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Profile topic listener started for contact [3mpeer_did[0m[2m=[0mdid:sync:z5CKkM9ErHpCxAG8ecUWUBtGfjmqbDFMpCt48Fh36oFEg
[2m2026-01-21T14:09:29.926623Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Adding peer address to static discovery [3mpeer[0m[2m=[0ma8ffc580058e8d9e771e2e63c916fc2b8b83014593e1a3d64991af80be746d07 [3maddrs[0m[2m=[0m6
[2m2026-01-21T14:09:29.926657Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(499ad0bfbf0d9446b65698325fe1a99d0fc682b884f4b953e65bf123651b8cff) [3mpeer_count[0m[2m=[0m1
[2m2026-01-21T14:09:29.926687Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Subscribed to contact gossip topic with profile listener [3mpeer_did[0m[2m=[0mdid:sync:zC89fuuhho1CbXMqhw974tM6XnfkpbRBwihaf5XMbCeMK [3mtopic_id[0m[2m=[0mTopicId(499ad0bfbf0d9446b65698325fe1a99d0fc682b884f4b953e65bf123651b8cff) [3malready_had_sender[0m[2m=[0mfalse
[2m2026-01-21T14:09:29.926708Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Adding peer address to static discovery [3mpeer[0m[2m=[0ma8ffc580058e8d9e771e2e63c916fc2b8b83014593e1a3d64991af80be746d07 [3maddrs[0m[2m=[0m6
[2m2026-01-21T14:09:29.926722Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(509084bceef34719c29d2ada6b81eb737629c10c70086f41accb1fa65619d5a2) [3mpeer_count[0m[2m=[0m1
[2m2026-01-21T14:09:29.926718Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Contact topic listener STARTED - attempting to receive events [3mpeer_did[0m[2m=[0mdid:sync:zC89fuuhho1CbXMqhw974tM6XnfkpbRBwihaf5XMbCeMK
[2m2026-01-21T14:09:29.926729Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Subscribed to contact's profile topic on reconnect [3mpeer_did[0m[2m=[0mdid:sync:zC89fuuhho1CbXMqhw974tM6XnfkpbRBwihaf5XMbCeMK [3mpeer_profile_topic[0m[2m=[0mTopicId(509084bceef34719c29d2ada6b81eb737629c10c70086f41accb1fa65619d5a2)
[2m2026-01-21T14:09:29.926765Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Profile topic listener started for contact [3mpeer_did[0m[2m=[0mdid:sync:zC89fuuhho1CbXMqhw974tM6XnfkpbRBwihaf5XMbCeMK
[2m2026-01-21T14:09:30.035576Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile announcement broadcast on contact topics [3mcontacts_updated[0m[2m=[0m2
[2m2026-01-21T14:09:30.035626Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile announcement broadcast complete
[2m2026-01-21T14:09:30.035633Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Presence announced on profile and contact topics
[2m2026-01-21T14:09:30.035705Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Attempting startup sync with known peers [3mpeer_count[0m[2m=[0m2
[2m2026-01-21T14:09:30.035732Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Startup sync complete [3mattempted[0m[2m=[0m0 [3msucceeded[0m[2m=[0m2 [3mskipped[0m[2m=[0m0 [3mjitter_ms[0m[2m=[0m950
[2m2026-01-21T14:09:30.035741Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m Startup sync complete: 2 succeeded, 0 attempted, 0 skipped (backoff), jitter=950ms
[2m2026-01-21T14:09:30.035885Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m SyncEngine initialized with identity
[2m2026-01-21T14:09:30.035992Z[0m [32m INFO[0m [2msyncengine_desktop::pages::landing[0m[2m:[0m Returning user detected (1 realms), auto-navigating to Field
[2m2026-01-21T14:09:30.049088Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Opening realm [3mrealm_id[0m[2m=[0mrealm_KQ33SsNGJWv
[2m2026-01-21T14:09:30.051466Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Loaded 5 tasks for realm Private
[2m2026-01-21T14:09:30.051477Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Total realms loaded: 1, total task entries: 1
[2m2026-01-21T14:09:30.051556Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Setting signals - realms: 1, tasks_by_realm entries: 1
[2m2026-01-21T14:09:30.051566Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Signals set - generation now: 1, data_loaded: true
[2m2026-01-21T14:09:30.051712Z[0m [32m INFO[0m [2msyncengine_desktop::components::unified_field[0m[2m:[0m UnifiedFieldView rendering - realms: 1, tasks_by_realm entries: 1, generation: 1
[2m2026-01-21T14:09:30.051724Z[0m [32m INFO[0m [2msyncengine_desktop::components::unified_field[0m[2m:[0m Rendering realm Private with 5 tasks
[2m2026-01-21T14:09:30.093824Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded historical packet events from storage [3mloaded_count[0m[2m=[0m2 [3mcontact_count[0m[2m=[0m2
```

</details>

---

## Session Summary

- **Ended:** 2026-01-21 14:09:58
- **Duration:** ~0 minutes

### Log Statistics

| Level | Count |
|-------|-------|
| INFO  | 59 |
| WARN  | 0
0 |
| ERROR | 0
0 |
