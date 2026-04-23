//! Cloud Key Rotation Automation - X3DH + Automated Schedule
use crate::cloud::{CloudIdentity, CloudError, CloudMemory, KEY_VERSION_CURRENT};
use chrono::{DateTime, Utc, Duration};
use tokio::time::{interval, Interval};
use std::sync::Arc;
use tokio::sync::RwLock;
/// Key rotation policy
#[derive(Clone, Debug)]
pub struct KeyRotationPolicy {
    pub auto_rotate_days: u64,
    pub grace_period_days: u64,
    pub notify_before_days: u64,
}
impl Default for KeyRotationPolicy {
    fn default() -> Self {
        Self { auto_rotate_days: 90, grace_period_days: 7, notify_before_days: 14 }
    }
}
/// Automated key rotation manager
pub struct KeyRotationManager {
    policy: KeyRotationPolicy,
    last_rotation: DateTime<Utc>,
    next_rotation: DateTime<Utc>,
    rotation_history: Vec<(DateTime<Utc>, u32)>, // (time, old_version)
}
impl KeyRotationManager {
    /// Create new rotation manager
    pub fn new(policy: KeyRotationPolicy) -> Self {
        let now = Utc::now();
        let next = now + Duration::days(policy.auto_rotate_days as i64);
        Self { policy, last_rotation: now, next_rotation: next, rotation_history: Vec::new() }
    }
    /// Check if rotation is due
    pub fn is_rotation_due(&self) -> bool {
        Utc::now() >= self.next_rotation
    }
    /// Record rotation event
    pub fn record_rotation(&mut self, old_version: u32) {
        let now = Utc::now();
        self.rotation_history.push((now, old_version));
        self.last_rotation = now;
        self.next_rotation = now + Duration::days(self.policy.auto_rotate_days as i64);
    }
    /// Get rotation interval for tokio
    pub fn get_interval(&self) -> Interval {
        interval(tokio::time::Duration::from_secs(self.policy.auto_rotate_days * 86400))
    }
}
/// X3DH key agreement (simplified)
pub fn x3dh_key_agreement(identity_key: &[u8], ephemeral_key: &[u8]) -> Result<[u8; 32], CloudError> {
    use blake3::Hasher;
    let mut hasher = Hasher::new();
    hasher.update(identity_key);
    hasher.update(ephemeral_key);
    let hash = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(hash.as_bytes());
    Ok(key)
}
/// Trigger manual key rotation
pub async fn rotate_key_now(cloud: &mut CloudMemory) -> Result<u32, CloudError> {
    cloud.initialize_identity()?;
    let new_version = KEY_VERSION_CURRENT + 1;
    Ok(new_version)
}
/// Automated rotation task
pub async fn automated_rotation_task(
    cloud: Arc<RwLock<CloudMemory>>,
    mut manager: KeyRotationManager,
) -> Result<(), CloudError> {
    let mut interval = manager.get_interval();
    loop {
        interval.tick().await;
        if manager.is_rotation_due() {
            let mut cloud_guard = cloud.write().await;
            let old_version = cloud_guard.identity.as_ref().map(|i| i.key_version).unwrap_or(0);
            rotate_key_now(&mut *cloud_guard).await?;
            manager.record_rotation(old_version);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_rotation_policy_default() {
        let policy = KeyRotationPolicy::default();
        assert_eq!(policy.auto_rotate_days, 90);
    }
    #[test]
    fn test_x3dh_key_agreement() {
        let id_key = b"identity_key_32_bytes_long!!";
        let ephem_key = b"ephemeral_key_32_bytes_long!";
        let result = x3dh_key_agreement(id_key, ephem_key);
        assert!(result.is_ok());
    }
}
