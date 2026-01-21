# Synchronicity Engine - Run Logs

**Started:** 2026-01-21 12:37:52

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
[2m2026-01-21T12:38:07.381049Z[0m [32m INFO[0m [2msyncengine_desktop[0m[2m:[0m Starting 'love' with data dir: "/Users/truman/Library/Application Support/instance-love", screen: 1512x982, window: 756x957, total_windows: 2
[2m2026-01-21T12:38:07.391522Z[0m [32m INFO[0m [2msyncengine_desktop[0m[2m:[0m Starting 'joy' with data dir: "/Users/truman/Library/Application Support/instance-joy", screen: 1512x982, window: 756x957, total_windows: 2
[2m2026-01-21T12:38:07.788596Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing SyncEngine [3mdata_dir[0m[2m=[0m"/Users/truman/Library/Application Support/instance-joy"
[2m2026-01-21T12:38:07.790574Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing SyncEngine [3mdata_dir[0m[2m=[0m"/Users/truman/Library/Application Support/instance-love"
[2m2026-01-21T12:38:07.859166Z[0m [32m INFO[0m [2msyncengine_core::blobs[0m[2m:[0m Creating persistent blob manager with FsStore [3mpath[0m[2m=[0m"/Users/truman/Library/Application Support/instance-joy/blobs"
[2m2026-01-21T12:38:07.863236Z[0m [32m INFO[0m [2msyncengine_core::blobs[0m[2m:[0m Creating persistent blob manager with FsStore [3mpath[0m[2m=[0m"/Users/truman/Library/Application Support/instance-love/blobs"
[2m2026-01-21T12:38:07.907246Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Blob manager initialized with persistent storage [3mblob_path[0m[2m=[0m"/Users/truman/Library/Application Support/instance-joy/blobs"
[2m2026-01-21T12:38:07.912916Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Blob manager initialized with persistent storage [3mblob_path[0m[2m=[0m"/Users/truman/Library/Application Support/instance-love/blobs"
[2m2026-01-21T12:38:07.917041Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded existing identity
[2m2026-01-21T12:38:07.917179Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded existing profile keys (DID matches identity)
[2m2026-01-21T12:38:07.917309Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded existing profile log from MirrorStore [3mpacket_count[0m[2m=[0m2 [3mhead_seq[0m[2m=[0mSome(1)
[2m2026-01-21T12:38:07.917422Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing contact manager
[2m2026-01-21T12:38:07.917452Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing gossip networking
[2m2026-01-21T12:38:07.917456Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded persistent endpoint secret key from storage
[2m2026-01-21T12:38:07.920810Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded existing identity
[2m2026-01-21T12:38:07.920870Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded existing profile keys (DID matches identity)
[2m2026-01-21T12:38:07.920927Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded existing profile log from MirrorStore [3mpacket_count[0m[2m=[0m1 [3mhead_seq[0m[2m=[0mSome(0)
[2m2026-01-21T12:38:07.920996Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing contact manager
[2m2026-01-21T12:38:07.921002Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Initializing gossip networking
[2m2026-01-21T12:38:07.921007Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded persistent endpoint secret key from storage
[2m2026-01-21T12:38:07.932147Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Endpoint bound [3mendpoint_id[0m[2m=[0m469d37b868b512d088786a0106fc313a7e04e60d0580c0bbda7dcae3d859654f
[2m2026-01-21T12:38:07.932147Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Endpoint bound [3mendpoint_id[0m[2m=[0m5083f0b5b1309974dca1facf6d3d50c930b7f864d84e20865e7fd2d5fcc4587c
[2m2026-01-21T12:38:07.932584Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Gossip spawned [3mmax_message_size[0m[2m=[0m1048576
[2m2026-01-21T12:38:07.932584Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Gossip spawned [3mmax_message_size[0m[2m=[0m1048576
[2m2026-01-21T12:38:07.932641Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Contact protocol handler registered with shared active_topics
[2m2026-01-21T12:38:07.932646Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Profile protocol handler registered
[2m2026-01-21T12:38:07.932652Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Contact protocol handler registered with shared active_topics
[2m2026-01-21T12:38:07.932655Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Profile protocol handler registered
[2m2026-01-21T12:38:07.932749Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Blob protocol handler registered for P2P image transfer
[2m2026-01-21T12:38:07.932779Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Router spawned
[2m2026-01-21T12:38:07.932869Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Blob protocol handler registered for P2P image transfer
[2m2026-01-21T12:38:07.932903Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Router spawned
[2m2026-01-21T12:38:07.937739Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Contact auto-accept task started
[2m2026-01-21T12:38:07.937745Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Contact subscription task started
[2m2026-01-21T12:38:07.937741Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Contact auto-accept task started
[2m2026-01-21T12:38:07.937748Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Contact subscription task started
[2m2026-01-21T12:38:07.937752Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Contact accepted profile announcer started
[2m2026-01-21T12:38:07.937752Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Contact accepted profile announcer started
[2m2026-01-21T12:38:07.937778Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(db58ccf0bf7d1c4d208a799d38d40dbd9132224f53354eedb849a0e66005b4c0) [3mpeer_count[0m[2m=[0m0
[2m2026-01-21T12:38:07.937781Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(88eaffe2b70adc79c946b2788e5302db91d7bda6d52e3504b7225cb2a630a634) [3mpeer_count[0m[2m=[0m0
[2m2026-01-21T12:38:07.937901Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Subscribed to own profile topic for broadcasting [3mdid[0m[2m=[0mdid:sync:z5CKkM9ErHpCxAG8ecUWUBtGfjmqbDFMpCt48Fh36oFEg [3mown_topic_id[0m[2m=[0mTopicId(88eaffe2b70adc79c946b2788e5302db91d7bda6d52e3504b7225cb2a630a634)
[2m2026-01-21T12:38:07.937909Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(fefc9e40f7d91e18f1948815ca1dbcfc6bd239bc4ecf7f7616c1dac94a05cb1a) [3mpeer_count[0m[2m=[0m0
[2m2026-01-21T12:38:07.937902Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Subscribed to own profile topic for broadcasting [3mdid[0m[2m=[0mdid:sync:z7r87Z6JF87gYZfFW2FVPgpskzQqnf32oTHtLmdoqYML3 [3mown_topic_id[0m[2m=[0mTopicId(db58ccf0bf7d1c4d208a799d38d40dbd9132224f53354eedb849a0e66005b4c0)
[2m2026-01-21T12:38:07.937908Z[0m [32m INFO[0m [2msyncengine_core::sync::gossip[0m[2m:[0m Subscribing to topic (split) [3mtopic_id[0m[2m=[0mTopicId(fefc9e40f7d91e18f1948815ca1dbcfc6bd239bc4ecf7f7616c1dac94a05cb1a) [3mpeer_count[0m[2m=[0m0
[2m2026-01-21T12:38:07.937921Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile sync started on global topic
[2m2026-01-21T12:38:07.937921Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile sync started on global topic
[2m2026-01-21T12:38:07.937993Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile sync listener started with P2P blob download support
[2m2026-01-21T12:38:07.938033Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile sync listener started with P2P blob download support
[2m2026-01-21T12:38:07.938253Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Generated hybrid contact invite (v2) [3minvite_id[0m[2m=[0m[192, 217, 14, 72, 185, 13, 102, 218, 72, 117, 157, 14, 164, 190, 207, 120] [3mversion[0m[2m=[0m2 [3mexpiry_hours[0m[2m=[0m24 [3moriginal_size[0m[2m=[0m243 [3mcompressed_size[0m[2m=[0m238 [3mcompression_ratio[0m[2m=[0m"102%" [3mfinal_length[0m[2m=[0m331
[2m2026-01-21T12:38:07.938254Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Generated hybrid contact invite (v2) [3minvite_id[0m[2m=[0m[134, 94, 190, 96, 141, 42, 108, 5, 68, 234, 150, 50, 68, 186, 191, 0] [3mversion[0m[2m=[0m2 [3mexpiry_hours[0m[2m=[0m24 [3moriginal_size[0m[2m=[0m245 [3mcompressed_size[0m[2m=[0m240 [3mcompression_ratio[0m[2m=[0m"102%" [3mfinal_length[0m[2m=[0m333
[2m2026-01-21T12:38:07.943401Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m Bootstrap invite written to "/Users/truman/Library/Application Support/syncengine-bootstrap/joy4.invite"
[2m2026-01-21T12:38:07.946841Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m Bootstrap invite written to "/Users/truman/Library/Application Support/syncengine-bootstrap/love 2.invite"
[2m2026-01-21T12:38:08.454149Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Saved outgoing contact request [3minvite_id[0m[2m=[0m[31, 196, 219, 133, 191, 191, 6, 97, 126, 28, 223, 246, 202, 135, 83, 30] [3mpeer_did[0m[2m=[0mdid:sync:z7r87Z6JF87gYZfFW2FVPgpskzQqnf32oTHtLmdoqYML3
[2m2026-01-21T12:38:08.459102Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Saved outgoing contact request [3minvite_id[0m[2m=[0m[31, 196, 219, 133, 191, 191, 6, 97, 126, 28, 223, 246, 202, 135, 83, 30] [3mpeer_did[0m[2m=[0mdid:sync:z7r87Z6JF87gYZfFW2FVPgpskzQqnf32oTHtLmdoqYML3
[2m2026-01-21T12:38:08.460663Z[0m [33m WARN[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Operation failed, retrying [3moperation[0m[2m=[0m"send_contact_request" [3mattempt[0m[2m=[0m1 [3mmax_attempts[0m[2m=[0m3 [3mdelay_ms[0m[2m=[0m100 [3merror[0m[2m=[0mNetwork error: Failed to connect to inviter: Connecting to ourself is not supported
[2m2026-01-21T12:38:08.562031Z[0m [33m WARN[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Operation failed, retrying [3moperation[0m[2m=[0m"send_contact_request" [3mattempt[0m[2m=[0m2 [3mmax_attempts[0m[2m=[0m3 [3mdelay_ms[0m[2m=[0m200 [3merror[0m[2m=[0mNetwork error: Failed to connect to inviter: Connecting to ourself is not supported
[2m2026-01-21T12:38:08.764155Z[0m [33m WARN[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Operation failed after all retries [3moperation[0m[2m=[0m"send_contact_request" [3mattempt[0m[2m=[0m3 [3merror[0m[2m=[0mNetwork error: Failed to connect to inviter: Connecting to ourself is not supported
[2m2026-01-21T12:38:08.771109Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Saved outgoing contact request [3minvite_id[0m[2m=[0m[245, 90, 226, 151, 69, 136, 99, 161, 107, 30, 66, 66, 18, 133, 206, 180] [3mpeer_did[0m[2m=[0mdid:sync:z5CKkM9ErHpCxAG8ecUWUBtGfjmqbDFMpCt48Fh36oFEg
[2m2026-01-21T12:38:09.901872Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Sent ContactRequest [3mpeer[0m[2m=[0m469d37b868b512d088786a0106fc313a7e04e60d0580c0bbda7dcae3d859654f [3minvite_id[0m[2m=[0m[245, 90, 226, 151, 69, 136, 99, 161, 107, 30, 66, 66, 18, 133, 206, 180]
[2m2026-01-21T12:38:09.914278Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_handler[0m[2m:[0m Received contact request, saved as IncomingPending [3minvite_id[0m[2m=[0m[245, 90, 226, 151, 69, 136, 99, 161, 107, 30, 66, 66, 18, 133, 206, 180] [3mrequester_did[0m[2m=[0mdid:sync:z7r87Z6JF87gYZfFW2FVPgpskzQqnf32oTHtLmdoqYML3 [3mauto_accept[0m[2m=[0mfalse
[2m2026-01-21T12:38:10.404174Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m Bootstrap: sent contact request to 'joy'
[2m2026-01-21T12:38:10.404262Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Starting startup sync...
[2m2026-01-21T12:38:10.844977Z[0m [33m WARN[0m [2msyncengine_core::engine[0m[2m:[0m Failed to reconnect contacts (non-fatal) [3merror[0m[2m=[0mSerialization error: Hit the end of buffer, expected more data
[2m2026-01-21T12:38:10.851492Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile announcement broadcast on contact topics [3mcontacts_updated[0m[2m=[0m0
[2m2026-01-21T12:38:10.851506Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile announcement broadcast complete
[2m2026-01-21T12:38:10.851511Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Presence announced on profile and contact topics
[2m2026-01-21T12:38:10.851574Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Attempting startup sync with known peers [3mpeer_count[0m[2m=[0m2
[2m2026-01-21T12:38:10.851598Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Startup sync complete [3mattempted[0m[2m=[0m0 [3msucceeded[0m[2m=[0m2 [3mskipped[0m[2m=[0m0 [3mjitter_ms[0m[2m=[0m440
[2m2026-01-21T12:38:10.851605Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m Startup sync complete: 2 succeeded, 0 attempted, 0 skipped (backoff), jitter=440ms
[2m2026-01-21T12:38:10.851629Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m SyncEngine initialized with identity
[2m2026-01-21T12:38:10.851773Z[0m [32m INFO[0m [2msyncengine_desktop::pages::landing[0m[2m:[0m Returning user detected (1 realms), auto-navigating to Field
[2m2026-01-21T12:38:10.860051Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Opening realm [3mrealm_id[0m[2m=[0mrealm_KQ33SsNGJWv
[2m2026-01-21T12:38:10.862219Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Loaded 5 tasks for realm Private
[2m2026-01-21T12:38:10.862258Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Total realms loaded: 1, total task entries: 1
[2m2026-01-21T12:38:10.862288Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Setting signals - realms: 1, tasks_by_realm entries: 1
[2m2026-01-21T12:38:10.862298Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Signals set - generation now: 1, data_loaded: true
[2m2026-01-21T12:38:10.862391Z[0m [32m INFO[0m [2msyncengine_desktop::components::unified_field[0m[2m:[0m UnifiedFieldView rendering - realms: 1, tasks_by_realm entries: 1, generation: 1
[2m2026-01-21T12:38:10.862403Z[0m [32m INFO[0m [2msyncengine_desktop::components::unified_field[0m[2m:[0m Rendering realm Private with 5 tasks
[2m2026-01-21T12:38:10.904744Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded historical packet events from storage [3mloaded_count[0m[2m=[0m2 [3mcontact_count[0m[2m=[0m2
[2m2026-01-21T12:38:11.458298Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Sent ContactRequest [3mpeer[0m[2m=[0m5083f0b5b1309974dca1facf6d3d50c930b7f864d84e20865e7fd2d5fcc4587c [3minvite_id[0m[2m=[0m[31, 196, 219, 133, 191, 191, 6, 97, 126, 28, 223, 246, 202, 135, 83, 30]
[2m2026-01-21T12:38:11.467319Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_handler[0m[2m:[0m Received contact request, saved as IncomingPending [3minvite_id[0m[2m=[0m[31, 196, 219, 133, 191, 191, 6, 97, 126, 28, 223, 246, 202, 135, 83, 30] [3mrequester_did[0m[2m=[0mdid:sync:z5CKkM9ErHpCxAG8ecUWUBtGfjmqbDFMpCt48Fh36oFEg [3mauto_accept[0m[2m=[0mfalse
[2m2026-01-21T12:38:11.960774Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m Bootstrap: sent contact request to 'love'
[2m2026-01-21T12:38:11.967275Z[0m [32m INFO[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Saved outgoing contact request [3minvite_id[0m[2m=[0m[245, 90, 226, 151, 69, 136, 99, 161, 107, 30, 66, 66, 18, 133, 206, 180] [3mpeer_did[0m[2m=[0mdid:sync:z5CKkM9ErHpCxAG8ecUWUBtGfjmqbDFMpCt48Fh36oFEg
[2m2026-01-21T12:38:11.968670Z[0m [33m WARN[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Operation failed, retrying [3moperation[0m[2m=[0m"send_contact_request" [3mattempt[0m[2m=[0m1 [3mmax_attempts[0m[2m=[0m3 [3mdelay_ms[0m[2m=[0m100 [3merror[0m[2m=[0mNetwork error: Failed to connect to inviter: Connecting to ourself is not supported
[2m2026-01-21T12:38:12.070529Z[0m [33m WARN[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Operation failed, retrying [3moperation[0m[2m=[0m"send_contact_request" [3mattempt[0m[2m=[0m2 [3mmax_attempts[0m[2m=[0m3 [3mdelay_ms[0m[2m=[0m200 [3merror[0m[2m=[0mNetwork error: Failed to connect to inviter: Connecting to ourself is not supported
[2m2026-01-21T12:38:12.272704Z[0m [33m WARN[0m [2msyncengine_core::sync::contact_manager[0m[2m:[0m Operation failed after all retries [3moperation[0m[2m=[0m"send_contact_request" [3mattempt[0m[2m=[0m3 [3merror[0m[2m=[0mNetwork error: Failed to connect to inviter: Connecting to ourself is not supported
[2m2026-01-21T12:38:12.272828Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Starting startup sync...
[2m2026-01-21T12:38:12.748875Z[0m [33m WARN[0m [2msyncengine_core::engine[0m[2m:[0m Failed to reconnect contacts (non-fatal) [3merror[0m[2m=[0mSerialization error: Hit the end of buffer, expected more data
[2m2026-01-21T12:38:12.756002Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile announcement broadcast on contact topics [3mcontacts_updated[0m[2m=[0m0
[2m2026-01-21T12:38:12.756011Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Profile announcement broadcast complete
[2m2026-01-21T12:38:12.756016Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Presence announced on profile and contact topics
[2m2026-01-21T12:38:12.756042Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Attempting startup sync with known peers [3mpeer_count[0m[2m=[0m2
[2m2026-01-21T12:38:12.756063Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Startup sync complete [3mattempted[0m[2m=[0m0 [3msucceeded[0m[2m=[0m2 [3mskipped[0m[2m=[0m0 [3mjitter_ms[0m[2m=[0m474
[2m2026-01-21T12:38:12.756068Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m Startup sync complete: 2 succeeded, 0 attempted, 0 skipped (backoff), jitter=474ms
[2m2026-01-21T12:38:12.756088Z[0m [32m INFO[0m [2msyncengine_desktop::app[0m[2m:[0m SyncEngine initialized with identity
[2m2026-01-21T12:38:12.756143Z[0m [32m INFO[0m [2msyncengine_desktop::pages::landing[0m[2m:[0m Returning user detected (1 realms), auto-navigating to Field
[2m2026-01-21T12:38:12.762796Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Opening realm [3mrealm_id[0m[2m=[0mrealm_4YB1GadZnTK
[2m2026-01-21T12:38:12.763298Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Loaded 5 tasks for realm Private
[2m2026-01-21T12:38:12.763308Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Total realms loaded: 1, total task entries: 1
[2m2026-01-21T12:38:12.763332Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Setting signals - realms: 1, tasks_by_realm entries: 1
[2m2026-01-21T12:38:12.763338Z[0m [32m INFO[0m [2msyncengine_desktop::pages::field[0m[2m:[0m Signals set - generation now: 1, data_loaded: true
[2m2026-01-21T12:38:12.763405Z[0m [32m INFO[0m [2msyncengine_desktop::components::unified_field[0m[2m:[0m UnifiedFieldView rendering - realms: 1, tasks_by_realm entries: 1, generation: 1
[2m2026-01-21T12:38:12.763419Z[0m [32m INFO[0m [2msyncengine_desktop::components::unified_field[0m[2m:[0m Rendering realm Private with 5 tasks
[2m2026-01-21T12:38:12.800339Z[0m [32m INFO[0m [2msyncengine_core::engine[0m[2m:[0m Loaded historical packet events from storage [3mloaded_count[0m[2m=[0m3 [3mcontact_count[0m[2m=[0m2
