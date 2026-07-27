//! Linear Capability Tracking: SSA form for capabilities
//!
//! Capabilities represent permissions to perform operations.
//! This module tracks the SSA form of capabilities, ensuring:
//! - Exactly one path can have write capability
//! - All capabilities are accounted for
//! - Capabilities are released when no longer needed

use std::collections::HashMap as StdHashMap;

/// A linear capability: permission to perform an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    Read,      // Permission to read
    Write,     // Permission to write (linear: exactly one holder)
    Deallocate, // Permission to deallocate
}

/// Tracks the SSA form of capabilities.
/// In SSA, each variable (here, capability) is assigned exactly once.
#[derive(Debug, Clone)]
pub struct CapabilitySSA {
    // capability_id -> (resource_id, capability_kind)
    assignments: StdHashMap<u32, (u32, Capability)>,
    // Tracks which capabilities are live at each program point
    live_capabilities: StdHashMap<u32, bool>, // capability_id -> live?
}

impl CapabilitySSA {
    pub fn new() -> Self {
        CapabilitySSA {
            assignments: Default::default(),
            live_capabilities: Default::default(),
        }
    }

    /// Assign a capability to a resource (once, in SSA form).
    pub fn assign_capability(
        &mut self,
        capability_id: u32,
        resource_id: u32,
        kind: Capability,
    ) -> Result<(), String> {
        if self.assignments.contains_key(&capability_id) {
            return Err(format!(
                "Capability {} already assigned (SSA violation)",
                capability_id
            ));
        }

        self.assignments.insert(capability_id, (resource_id, kind));
        self.live_capabilities.insert(capability_id, true);
        Ok(())
    }

    /// Mark a capability as no longer live (released).
    pub fn release_capability(&mut self, capability_id: u32) -> Result<(), String> {
        if !self.live_capabilities.contains_key(&capability_id) {
            return Err(format!("Capability {} not found", capability_id));
        }

        self.live_capabilities.insert(capability_id, false);
        Ok(())
    }

    /// Verify that write capabilities are linear (at most one holder).
    pub fn verify_write_linearity(&self, resource_id: u32) -> Result<(), String> {
        let write_holders: Vec<u32> = self
            .assignments
            .iter()
            .filter(|(_, (res_id, kind))| *res_id == resource_id && *kind == Capability::Write)
            .map(|(cap_id, _)| *cap_id)
            .collect();

        if write_holders.len() > 1 {
            return Err(format!(
                "Multiple write capabilities for resource {}: {:?}",
                resource_id, write_holders
            ));
        }

        Ok(())
    }

    /// Verify that all capabilities are released at function end.
    pub fn verify_all_released(&self) -> Result<(), Vec<u32>> {
        let unreleased: Vec<u32> = self
            .live_capabilities
            .iter()
            .filter(|(_, &live)| live)
            .map(|(&cap_id, _)| cap_id)
            .collect();

        if unreleased.is_empty() {
            Ok(())
        } else {
            Err(unreleased)
        }
    }

    pub fn get_capability(&self, capability_id: u32) -> Option<(u32, Capability)> {
        self.assignments.get(&capability_id).copied()
    }

    pub fn all_assignments(&self) -> &StdHashMap<u32, (u32, Capability)> {
        &self.assignments
    }
}

// TODO: Implement capability inference: automatically assign capabilities based on RMIR instructions
// TODO: Implement split/join of capabilities (borrow splits write, borrow end joins back)
// TODO: Implement capability transfer (move, borrow, reborrow)
// TODO: Implement proof obligations for capability linearity
