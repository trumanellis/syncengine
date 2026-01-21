//! Profile protocol for public profile serving
//!
//! This module implements a simple request-response protocol for fetching
//! public profiles from peers. This enables compact contact invites that fetch
//! full profile data on-demand when the inviter is online.
//!
//! ## Protocol Overview
//!
//! The profile protocol provides a simple query interface for peer profiles:
//!
//! 1. **GetProfile**: Requester asks for peer's profile
//! 2. **ProfileResponse**: Peer responds with signed profile data
//!
//! ## Message Flow
//!
//! ```text
//! Requester (Joy)               Profile Host (Love)
//!   |                               |
//!   |--- GetProfile --------------->|
//!   |                               |
//!   |<-- ProfileResponse -----------|
//!   |    (signed profile)           |
//!   |                               |
//! ```
//!
//! ## Security
//!
//! - Profiles are signed with HybridSignature to prove authenticity
//! - Requester verifies signature matches expected DID
//! - Profile data is always current (no stale snapshots)

use std::sync::Arc;

use iroh::endpoint::Connection;
use iroh::protocol::ProtocolHandler;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::error::SyncError;
use crate::identity::{Did, HybridKeypair};
use crate::storage::Storage;

/// ALPN protocol identifier for profile serving
///
/// Profile requests use a separate ALPN to allow independent protocol
/// evolution and resource allocation from contact exchange.
pub const PROFILE_ALPN: &[u8] = b"/sync/profile/1";

/// Profile protocol messages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProfileMessage {
    /// Request public profile from peer
    GetProfile,

    /// Response with signed profile data
    ProfileResponse {
        /// Public profile data
        profile: PublicProfile,
        /// HybridSignature over profile
        signature: Vec<u8>,
    },

    /// Error response
    Error {
        /// Error reason
        reason: String,
    },
}

impl ProfileMessage {
    /// Encode message to bytes using postcard
    pub fn encode(&self) -> Result<Vec<u8>, SyncError> {
        postcard::to_allocvec(self)
            .map_err(|e| SyncError::Serialization(format!("Failed to encode profile message: {}", e)))
    }

    /// Decode message from bytes using postcard
    pub fn decode(bytes: &[u8]) -> Result<Self, SyncError> {
        postcard::from_bytes(bytes)
            .map_err(|e| SyncError::Serialization(format!("Failed to decode profile message: {}", e)))
    }
}

/// Public profile data served over PROFILE_ALPN
///
/// This is a sanitized version of UserProfile suitable for public viewing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublicProfile {
    /// DID of the profile owner
    pub did: String,
    /// Display name
    pub display_name: String,
    /// Optional subtitle (role, tagline, etc.)
    pub subtitle: Option<String>,
    /// Full biography text
    pub bio: String,
    /// Iroh blob ID for avatar (if set)
    pub avatar_blob_id: Option<String>,
    /// Unix timestamp when profile was last updated
    pub updated_at: i64,
}

/// Protocol handler for serving public profiles
///
/// This is registered with the Router to handle incoming PROFILE_ALPN connections.
/// It serves the node's own profile to any requester (public data).
#[derive(Clone)]
pub struct ProfileProtocolHandler {
    storage: Arc<Storage>,
    keypair: Arc<HybridKeypair>,
    did: Did,
}

impl std::fmt::Debug for ProfileProtocolHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProfileProtocolHandler")
            .field("storage", &"<Storage>")
            .field("keypair", &"<HybridKeypair>")
            .field("did", &self.did.to_string())
            .finish()
    }
}

impl ProfileProtocolHandler {
    /// Create a new profile protocol handler
    pub fn new(
        storage: Arc<Storage>,
        keypair: Arc<HybridKeypair>,
        did: Did,
    ) -> Self {
        Self {
            storage,
            keypair,
            did,
        }
    }

    /// Get the ALPN identifier for this protocol
    pub const fn alpn() -> &'static [u8] {
        PROFILE_ALPN
    }

    /// Handle a profile request connection
    ///
    /// This processes the profile request and returns signed profile data.
    async fn handle_connection(
        connection: Connection,
        storage: Arc<Storage>,
        keypair: Arc<HybridKeypair>,
        did: Did,
    ) -> Result<(), SyncError> {
        let remote_id = connection.remote_id();
        debug!(?remote_id, "Handling profile request connection");

        // Accept a bi-directional stream
        let (mut send, mut recv) = connection
            .accept_bi()
            .await
            .map_err(|e| SyncError::Network(format!("Failed to accept bi stream: {}", e)))?;

        // Read the request
        let request_bytes = recv
            .read_to_end(1024) // Small request
            .await
            .map_err(|e| SyncError::Network(format!("Failed to read request: {}", e)))?;

        // Decode the message
        let message = ProfileMessage::decode(&request_bytes)?;
        debug!(?message, "Received profile message");

        // Process based on message type
        let response = match message {
            ProfileMessage::GetProfile => {
                info!(did = %did, "Serving public profile");

                // Load own profile from storage using DID as peer_id
                match storage.load_profile(did.as_str()) {
                    Ok(Some(user_profile)) => {
                        let public_profile = PublicProfile {
                            did: did.to_string(),
                            display_name: user_profile.display_name.clone(),
                            subtitle: user_profile.subtitle.clone(),
                            bio: user_profile.bio.clone(),
                            avatar_blob_id: user_profile.avatar_blob_id.clone(),
                            updated_at: chrono::Utc::now().timestamp(),
                        };

                        // Sign the profile
                        let profile_bytes = postcard::to_allocvec(&public_profile).map_err(|e| {
                            SyncError::Serialization(format!("Failed to serialize profile: {}", e))
                        })?;

                        let signature = keypair.sign(&profile_bytes);
                        let signature_bytes = signature.to_bytes();

                        debug!(
                            did = %did,
                            profile_size = profile_bytes.len(),
                            signature_size = signature_bytes.len(),
                            "Signed public profile"
                        );

                        ProfileMessage::ProfileResponse {
                            profile: public_profile,
                            signature: signature_bytes,
                        }
                    }
                    Ok(None) => {
                        warn!(did = %did, "No profile found in storage");
                        ProfileMessage::Error {
                            reason: "Profile not found".to_string(),
                        }
                    }
                    Err(e) => {
                        error!(did = %did, error = ?e, "Failed to load profile");
                        ProfileMessage::Error {
                            reason: format!("Storage error: {}", e),
                        }
                    }
                }
            }
            _ => ProfileMessage::Error {
                reason: "Unexpected message type".to_string(),
            },
        };

        // Send response
        let response_bytes = response.encode()?;
        send.write_all(&response_bytes)
            .await
            .map_err(|e| SyncError::Network(format!("Failed to write response: {}", e)))?;

        send.finish()
            .map_err(|e| SyncError::Network(format!("Failed to finish stream: {}", e)))?;

        info!(
            remote_id = ?remote_id,
            response_size = response_bytes.len(),
            "Profile request handled successfully"
        );

        Ok(())
    }
}

impl ProtocolHandler for ProfileProtocolHandler {
    fn accept(
        &self,
        conn: Connection,
    ) -> impl std::future::Future<Output = Result<(), iroh::protocol::AcceptError>> + Send {
        let storage = self.storage.clone();
        let keypair = self.keypair.clone();
        let did = self.did.clone();

        async move {
            debug!(peer = %conn.remote_id(), "Router accepting profile connection");

            // Process the connection fully before returning
            if let Err(e) = Self::handle_connection(conn, storage, keypair, did).await {
                error!(error = ?e, "Failed to handle profile connection");
                return Err(iroh::protocol::AcceptError::from_err(e));
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_message_encode_decode() {
        let message = ProfileMessage::GetProfile;
        let bytes = message.encode().unwrap();
        let decoded = ProfileMessage::decode(&bytes).unwrap();
        assert_eq!(message, decoded);
    }

    #[test]
    fn test_profile_response_encode_decode() {
        let profile = PublicProfile {
            did: "did:sync:z123".to_string(),
            display_name: "Test User".to_string(),
            subtitle: Some("Tester".to_string()),
            bio: "Testing the system".to_string(),
            avatar_blob_id: None,
            updated_at: 1234567890,
        };

        let message = ProfileMessage::ProfileResponse {
            profile: profile.clone(),
            signature: vec![1, 2, 3, 4],
        };

        let bytes = message.encode().unwrap();
        let decoded = ProfileMessage::decode(&bytes).unwrap();

        if let ProfileMessage::ProfileResponse {
            profile: decoded_profile,
            ..
        } = decoded
        {
            assert_eq!(decoded_profile.did, profile.did);
            assert_eq!(decoded_profile.display_name, profile.display_name);
        } else {
            panic!("Expected ProfileResponse");
        }
    }
}
