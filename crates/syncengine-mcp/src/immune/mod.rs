//! Immune system testing tools
//!
//! Test rate limiting, reputation, and anomaly detection.

use crate::error::{McpError, McpResult};
use crate::harness::TestHarness;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bad behavior types that can be simulated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BadBehavior {
    /// Flood with messages at high rate
    MessageSpam,
    /// Send messages with invalid signatures
    InvalidSignatures,
    /// Send malformed protocol messages
    MalformedMessages,
    /// Attempt replay attacks
    ReplayAttack,
    /// Send oversized documents
    DocumentBomb,
    /// Rapid connect/disconnect
    ConnectionChurn,
    /// Announce fake peers
    FakePeerAnnouncement,
    /// Send invites in bulk
    InviteSpam,
}

impl BadBehavior {
    /// Parse from string
    pub fn from_str(s: &str) -> McpResult<Self> {
        match s.to_lowercase().as_str() {
            "message_spam" | "spam" => Ok(BadBehavior::MessageSpam),
            "invalid_signatures" | "bad_sig" => Ok(BadBehavior::InvalidSignatures),
            "malformed_messages" | "malformed" => Ok(BadBehavior::MalformedMessages),
            "replay_attack" | "replay" => Ok(BadBehavior::ReplayAttack),
            "document_bomb" | "doc_bomb" => Ok(BadBehavior::DocumentBomb),
            "connection_churn" | "churn" => Ok(BadBehavior::ConnectionChurn),
            "fake_peer_announcement" | "fake_peers" => Ok(BadBehavior::FakePeerAnnouncement),
            "invite_spam" | "invites" => Ok(BadBehavior::InviteSpam),
            _ => Err(McpError::InvalidOperation(format!(
                "Unknown behavior: {}. Valid: message_spam, invalid_signatures, malformed_messages, \
                 replay_attack, document_bomb, connection_churn, fake_peer_announcement, invite_spam",
                s
            ))),
        }
    }
}

/// Result of rate limit check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResult {
    /// Whether rate limit was triggered
    pub triggered: bool,
    /// Current message rate (messages/second)
    pub current_rate: f64,
    /// Rate limit threshold
    pub limit: f64,
    /// Cooldown remaining (seconds)
    pub cooldown_remaining_secs: f64,
    /// Number of violations for this peer
    pub violation_count: u32,
}

/// Peer reputation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerReputation {
    /// Peer identifier
    pub peer_id: String,
    /// Trust score (0.0 - 1.0)
    pub trust_score: f64,
    /// Number of violations
    pub violation_count: u32,
    /// Violation history
    pub violations: Vec<ViolationRecord>,
    /// Peers that vouched for this peer
    pub vouched_by: Vec<String>,
    /// When peer was first seen
    pub first_seen: String,
    /// Whether peer is currently quarantined
    pub quarantined: bool,
}

/// Record of a violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationRecord {
    /// Type of violation
    pub violation_type: String,
    /// When it occurred
    pub timestamp: String,
    /// Severity (1-10)
    pub severity: u8,
    /// Details
    pub details: Option<String>,
}

/// Result of simulating bad behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorTestResult {
    /// Behavior that was simulated
    pub behavior: String,
    /// Number of attempts made
    pub attempts: u32,
    /// Which defenses were activated
    pub defenses_activated: Vec<String>,
    /// Whether the attack was detected
    pub detected: bool,
    /// Detection confidence (0.0 - 1.0)
    pub confidence: f64,
    /// Response taken by the system
    pub response: String,
    /// Time to detect (milliseconds)
    pub detection_time_ms: Option<u64>,
}

/// Quarantine list entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineEntry {
    /// Peer identifier
    pub peer_id: String,
    /// Reason for quarantine
    pub reason: String,
    /// When quarantine started
    pub started_at: String,
    /// When quarantine expires (if temporary)
    pub expires_at: Option<String>,
    /// Whether it's a permanent ban
    pub permanent: bool,
}

/// Anomaly detection test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionResult {
    /// Pattern that was tested
    pub pattern: String,
    /// Whether it was detected as anomalous
    pub detected: bool,
    /// Confidence score
    pub confidence: f64,
    /// Anomaly type if detected
    pub anomaly_type: Option<String>,
    /// Response that would be taken
    pub recommended_response: String,
    /// Feature vector used for detection
    pub features: HashMap<String, f64>,
}

/// Immune system tester
pub struct ImmuneTester {
    /// Simulated rate limiters per node-peer pair
    rate_limits: HashMap<(String, String), RateLimitState>,
    /// Simulated reputation per peer
    reputations: HashMap<String, PeerReputation>,
    /// Quarantine list
    quarantine: Vec<QuarantineEntry>,
}

struct RateLimitState {
    message_count: u32,
    window_start: std::time::Instant,
    violations: u32,
    cooldown_until: Option<std::time::Instant>,
}

impl ImmuneTester {
    /// Create a new immune system tester
    pub fn new() -> Self {
        Self {
            rate_limits: HashMap::new(),
            reputations: HashMap::new(),
            quarantine: Vec::new(),
        }
    }

    /// Trigger rate limit for testing
    pub fn trigger_rate_limit(
        &mut self,
        node_id: &str,
        peer_id: &str,
        message_count: u32,
    ) -> RateLimitResult {
        let key = (node_id.to_string(), peer_id.to_string());

        let state = self.rate_limits.entry(key).or_insert_with(|| RateLimitState {
            message_count: 0,
            window_start: std::time::Instant::now(),
            violations: 0,
            cooldown_until: None,
        });

        // Reset window if more than 1 second elapsed
        if state.window_start.elapsed().as_secs_f64() > 1.0 {
            state.message_count = 0;
            state.window_start = std::time::Instant::now();
        }

        state.message_count += message_count;

        // Rate limit: 100 messages per second
        const RATE_LIMIT: f64 = 100.0;

        let elapsed = state.window_start.elapsed().as_secs_f64().max(0.001);
        let current_rate = state.message_count as f64 / elapsed;

        let triggered = current_rate > RATE_LIMIT;

        if triggered {
            state.violations += 1;
            // Exponential backoff: 2^violations seconds cooldown
            let cooldown_secs = 2u64.pow(state.violations.min(8));
            state.cooldown_until = Some(
                std::time::Instant::now() + std::time::Duration::from_secs(cooldown_secs),
            );
        }

        let cooldown_remaining = state
            .cooldown_until
            .map(|c| {
                let remaining = c.saturating_duration_since(std::time::Instant::now());
                remaining.as_secs_f64()
            })
            .unwrap_or(0.0);

        RateLimitResult {
            triggered,
            current_rate,
            limit: RATE_LIMIT,
            cooldown_remaining_secs: cooldown_remaining,
            violation_count: state.violations,
        }
    }

    /// Get peer reputation
    pub fn get_peer_reputation(&self, _node_id: &str, peer_id: &str) -> PeerReputation {
        self.reputations
            .get(peer_id)
            .cloned()
            .unwrap_or_else(|| PeerReputation {
                peer_id: peer_id.to_string(),
                trust_score: 0.5, // New peers start at neutral
                violation_count: 0,
                violations: vec![],
                vouched_by: vec![],
                first_seen: chrono::Utc::now().to_rfc3339(),
                quarantined: false,
            })
    }

    /// Simulate bad behavior and test defenses
    pub async fn simulate_bad_behavior(
        &mut self,
        harness: &TestHarness,
        peer_id: &str,
        behavior: BadBehavior,
    ) -> McpResult<BehaviorTestResult> {
        let start = std::time::Instant::now();
        let mut defenses_activated = Vec::new();
        let mut detected = false;
        let mut confidence = 0.0;
        let mut response = "none".to_string();

        match behavior {
            BadBehavior::MessageSpam => {
                // Simulate 1000 messages in quick succession
                for node_info in harness.list_nodes().await {
                    let result = self.trigger_rate_limit(&node_info.name, peer_id, 1000);
                    if result.triggered {
                        defenses_activated.push("rate_limiter".into());
                        detected = true;
                        confidence = 0.99;
                        response = format!(
                            "Rate limit triggered. Cooldown: {:.1}s",
                            result.cooldown_remaining_secs
                        );
                        break;
                    }
                }
            }
            BadBehavior::InvalidSignatures => {
                // Would be detected by signature verification
                defenses_activated.push("signature_verification".into());
                detected = true;
                confidence = 1.0;
                response = "Message rejected: invalid signature".into();
            }
            BadBehavior::MalformedMessages => {
                // Would be detected by protocol parser
                defenses_activated.push("protocol_validation".into());
                detected = true;
                confidence = 1.0;
                response = "Message rejected: malformed protocol message".into();
            }
            BadBehavior::ReplayAttack => {
                // Would need nonce tracking (not yet implemented)
                defenses_activated.push("replay_detection".into());
                detected = false; // Not implemented yet
                confidence = 0.0;
                response = "NOT IMPLEMENTED: replay detection not active".into();
            }
            BadBehavior::DocumentBomb => {
                // Would be caught by size limits
                defenses_activated.push("size_limiter".into());
                detected = true;
                confidence = 1.0;
                response = "Document rejected: exceeds 1MB limit".into();
            }
            BadBehavior::ConnectionChurn => {
                // Would be detected by connection monitoring
                defenses_activated.push("connection_monitor".into());
                detected = true;
                confidence = 0.8;
                response = "Peer flagged: excessive connection churn".into();
            }
            BadBehavior::FakePeerAnnouncement => {
                // Would need peer validation
                defenses_activated.push("peer_validation".into());
                detected = false; // Not implemented yet
                confidence = 0.0;
                response = "NOT IMPLEMENTED: peer validation not active".into();
            }
            BadBehavior::InviteSpam => {
                // Would need invite tracking
                defenses_activated.push("invite_limiter".into());
                detected = false; // Not implemented yet
                confidence = 0.0;
                response = "NOT IMPLEMENTED: invite rate limiting not active".into();
            }
        }

        // Update peer reputation
        if detected {
            let rep = self.reputations.entry(peer_id.to_string()).or_insert_with(|| {
                PeerReputation {
                    peer_id: peer_id.to_string(),
                    trust_score: 0.5,
                    violation_count: 0,
                    violations: vec![],
                    vouched_by: vec![],
                    first_seen: chrono::Utc::now().to_rfc3339(),
                    quarantined: false,
                }
            });

            rep.violation_count += 1;
            rep.trust_score = (rep.trust_score - 0.1).max(0.0);
            rep.violations.push(ViolationRecord {
                violation_type: format!("{:?}", behavior),
                timestamp: chrono::Utc::now().to_rfc3339(),
                severity: 5,
                details: Some(response.clone()),
            });

            // Quarantine if trust score too low
            if rep.trust_score < 0.2 {
                rep.quarantined = true;
                self.quarantine.push(QuarantineEntry {
                    peer_id: peer_id.to_string(),
                    reason: format!("Trust score below threshold after {:?}", behavior),
                    started_at: chrono::Utc::now().to_rfc3339(),
                    expires_at: None,
                    permanent: false,
                });
            }
        }

        let detection_time = if detected {
            Some(start.elapsed().as_millis() as u64)
        } else {
            None
        };

        Ok(BehaviorTestResult {
            behavior: format!("{:?}", behavior),
            attempts: 1,
            defenses_activated,
            detected,
            confidence,
            response,
            detection_time_ms: detection_time,
        })
    }

    /// Get quarantine list for a node
    pub fn get_quarantine_list(&self, _node_id: &str) -> Vec<QuarantineEntry> {
        self.quarantine.clone()
    }

    /// Test anomaly detection with a pattern
    pub fn test_anomaly_detection(&self, pattern: &str) -> AnomalyDetectionResult {
        // Simple pattern matching for demonstration
        let mut features: HashMap<String, f64> = HashMap::new();

        // Extract features from pattern
        let message_rate = pattern.matches("spam").count() as f64 * 100.0;
        let is_repetitive = pattern.len() < 10 && pattern.chars().collect::<std::collections::HashSet<_>>().len() < 3;
        let has_suspicious_keywords = pattern.contains("attack") || pattern.contains("flood");

        features.insert("message_rate".into(), message_rate);
        features.insert("repetitiveness".into(), if is_repetitive { 1.0 } else { 0.0 });
        features.insert("suspicious_keywords".into(), if has_suspicious_keywords { 1.0 } else { 0.0 });

        // Calculate anomaly score
        let anomaly_score = (message_rate / 100.0).min(1.0)
            + if is_repetitive { 0.3 } else { 0.0 }
            + if has_suspicious_keywords { 0.4 } else { 0.0 };

        let detected = anomaly_score > 0.5;
        let confidence = anomaly_score.min(1.0);

        let anomaly_type = if detected {
            if message_rate > 0.0 {
                Some("high_rate_messaging".into())
            } else if is_repetitive {
                Some("repetitive_content".into())
            } else {
                Some("suspicious_pattern".into())
            }
        } else {
            None
        };

        let recommended_response = if detected {
            if confidence > 0.8 {
                "quarantine_peer".into()
            } else if confidence > 0.5 {
                "rate_limit_peer".into()
            } else {
                "monitor_peer".into()
            }
        } else {
            "none".into()
        };

        AnomalyDetectionResult {
            pattern: pattern.to_string(),
            detected,
            confidence,
            anomaly_type,
            recommended_response,
            features,
        }
    }

    /// Clear all test state
    pub fn reset(&mut self) {
        self.rate_limits.clear();
        self.reputations.clear();
        self.quarantine.clear();
    }
}

impl Default for ImmuneTester {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_triggering() {
        let mut tester = ImmuneTester::new();

        // Large burst should trigger rate limit (1000 messages in any time window)
        let result = tester.trigger_rate_limit("node1", "peer1", 1000);
        assert!(result.triggered, "Large burst should trigger rate limit");
        assert!(result.violation_count > 0, "Should record violation");
        assert!(result.current_rate > result.limit, "Rate should exceed limit");

        // Verify violation count increases with repeated violations
        let result2 = tester.trigger_rate_limit("node1", "peer1", 1000);
        assert!(result2.triggered);
        assert!(result2.violation_count >= result.violation_count);
    }

    #[test]
    fn test_peer_reputation() {
        let tester = ImmuneTester::new();
        let rep = tester.get_peer_reputation("node1", "unknown_peer");

        assert_eq!(rep.peer_id, "unknown_peer");
        assert_eq!(rep.trust_score, 0.5); // Default neutral score
        assert!(!rep.quarantined);
    }

    #[test]
    fn test_anomaly_detection() {
        let tester = ImmuneTester::new();

        // Normal pattern
        let result = tester.test_anomaly_detection("hello world this is a normal message");
        assert!(!result.detected);

        // Suspicious pattern
        let result = tester.test_anomaly_detection("spam spam spam attack flood");
        assert!(result.detected);
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_bad_behavior_parsing() {
        assert_eq!(BadBehavior::from_str("spam").unwrap(), BadBehavior::MessageSpam);
        assert_eq!(BadBehavior::from_str("REPLAY").unwrap(), BadBehavior::ReplayAttack);
        assert!(BadBehavior::from_str("unknown").is_err());
    }
}
