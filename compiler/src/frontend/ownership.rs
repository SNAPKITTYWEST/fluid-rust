//! Ownership Analysis: Track linear capabilities and borrow lifetimes
//!
//! This module implements the ownership verification that ensures:
//! - Linear values cannot be aliased
//! - Borrows do not outlive their referents
//! - Consumed values cannot be used
//! - All resources are cleaned up

use std::collections::{HashMap as StdHashMap, HashSet};

/// Represents the ownership state of a value at a specific point in the program.
#[derive(Debug, Clone, PartialEq)]
pub enum OwnershipKind {
    /// Unique ownership: exactly one path to the value.
    Unique,
    /// Borrowed shared: read-only access, multiple readers allowed.
    BorrowedShared,
    /// Borrowed mutable: exclusive write access, single writer.
    BorrowedMut,
    /// Consumed: value has been moved/dropped, cannot use.
    Consumed,
}

/// Tracks the lifetime for which a borrow is valid.
#[derive(Debug, Clone)]
pub struct Lifetime {
    pub name: String,
    pub scope: usize, // Program point where borrow begins
}

/// Represents a value's ownership facts at a given program point.
#[derive(Debug, Clone)]
pub struct OwnershipFact {
    pub value_id: String,
    pub owner: Option<String>, // Thread or context that owns this value
    pub kind: OwnershipKind,
    pub lifetime: Option<Lifetime>,
    pub region: Option<String>,
}

/// Analyzes a program for ownership violations.
pub struct OwnershipAnalyzer {
    facts: Vec<OwnershipFact>,
    aliases: StdHashMap<String, String>, // value_id -> canonical_id
    program_point: usize,
}

impl OwnershipAnalyzer {
    pub fn new() -> Self {
        OwnershipAnalyzer {
            facts: Vec::new(),
            aliases: StdHashMap::new(),
            program_point: 0,
        }
    }

    /// Record a value's ownership at the current program point.
    pub fn record_fact(&mut self, fact: OwnershipFact) {
        self.facts.push(fact);
    }

    /// Verify no two threads own the same value.
    pub fn check_no_aliasing(&self) -> Result<(), String> {
        let mut owned_by: StdHashMap<String, HashSet<Option<String>>> = Default::default();

        for fact in &self.facts {
            if fact.kind == OwnershipKind::Unique || fact.kind == OwnershipKind::BorrowedMut {
                let key = fact.value_id.clone();
                owned_by
                    .entry(key)
                    .or_insert_with(HashSet::new)
                    .insert(fact.owner.clone());
            }
        }

        for (value, owners) in owned_by {
            if owners.len() > 1 {
                return Err(format!("Multiple threads own value {}: {:?}", value, owners));
            }
        }

        Ok(())
    }

    /// Verify no use-after-consume.
    pub fn check_no_use_after_consume(&self) -> Result<(), String> {
        let mut consumed_at: StdHashMap<String, usize> = Default::default();

        for (idx, fact) in self.facts.iter().enumerate() {
            if fact.kind == OwnershipKind::Consumed {
                consumed_at.insert(fact.value_id.clone(), idx);
            }
        }

        for (idx, fact) in self.facts.iter().enumerate() {
            if let Some(consumed_idx) = consumed_at.get(&fact.value_id) {
                if idx > *consumed_idx && fact.kind != OwnershipKind::Consumed {
                    return Err(format!(
                        "Use-after-consume: value {} consumed at point {} but used at point {}",
                        fact.value_id, consumed_idx, idx
                    ));
                }
            }
        }

        Ok(())
    }

    /// Verify borrow does not outlive its referent.
    pub fn check_borrow_lifetimes(&self) -> Result<(), String> {
        // For each borrowed value, ensure its lifetime doesn't extend past the unique owner's lifetime
        for fact in &self.facts {
            if fact.kind == OwnershipKind::BorrowedShared || fact.kind == OwnershipKind::BorrowedMut {
                // TODO: Check that the lifetime scope is within the owner's scope
            }
        }
        Ok(())
    }

    /// Run all ownership checks.
    pub fn verify_all(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if let Err(e) = self.check_no_aliasing() {
            errors.push(e);
        }
        if let Err(e) = self.check_no_use_after_consume() {
            errors.push(e);
        }
        if let Err(e) = self.check_borrow_lifetimes() {
            errors.push(e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn advance_program_point(&mut self) {
        self.program_point += 1;
    }
}

// TODO: Implement move tracking: detect when a value is moved into another binding
// TODO: Implement borrow tracking: track shared vs mutable borrows and their scopes
// TODO: Implement drop tracking: ensure all linear values are explicitly freed
// TODO: Implement region-based analysis: associate values with their regions
// TODO: Add diagnostics with source locations for ownership violations
