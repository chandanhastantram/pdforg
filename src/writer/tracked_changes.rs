//! Tracked changes — records of insertions, deletions, and format changes for review.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use super::ot::Op;

/// A tracked change applied to a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub id: Uuid,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub kind: ChangeKind,
    pub ops: Vec<Op>,
    pub status: ChangeStatus,
    pub comment: Option<String>,
}

impl Change {
    pub fn new(author: impl Into<String>, kind: ChangeKind, ops: Vec<Op>) -> Self {
        Change {
            id: Uuid::new_v4(),
            author: author.into(),
            timestamp: Utc::now(),
            kind,
            ops,
            status: ChangeStatus::Pending,
            comment: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChangeKind {
    Insertion,
    Deletion,
    FormatChange,
    Move { from_pos: usize, to_pos: usize },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChangeStatus {
    Pending,
    Accepted,
    Rejected,
}

/// Manages a collection of tracked changes for a document
#[derive(Debug, Default)]
pub struct ChangeTracker {
    pub changes: Vec<Change>,
    pub tracking_enabled: bool,
    pub current_author: String,
}

impl ChangeTracker {
    pub fn new(author: impl Into<String>) -> Self {
        ChangeTracker {
            changes: vec![],
            tracking_enabled: true,
            current_author: author.into(),
        }
    }

    pub fn record(&mut self, kind: ChangeKind, ops: Vec<Op>) -> Uuid {
        let change = Change::new(&self.current_author, kind, ops);
        let id = change.id;
        self.changes.push(change);
        id
    }

    pub fn accept(&mut self, id: Uuid) -> bool {
        if let Some(c) = self.changes.iter_mut().find(|c| c.id == id) {
            c.status = ChangeStatus::Accepted;
            true
        } else {
            false
        }
    }

    pub fn reject(&mut self, id: Uuid) -> bool {
        if let Some(c) = self.changes.iter_mut().find(|c| c.id == id) {
            c.status = ChangeStatus::Rejected;
            true
        } else {
            false
        }
    }

    pub fn accept_all(&mut self) {
        for c in &mut self.changes {
            if c.status == ChangeStatus::Pending {
                c.status = ChangeStatus::Accepted;
            }
        }
    }

    pub fn reject_all(&mut self) {
        for c in &mut self.changes {
            if c.status == ChangeStatus::Pending {
                c.status = ChangeStatus::Rejected;
            }
        }
    }

    pub fn pending(&self) -> Vec<&Change> {
        self.changes.iter().filter(|c| c.status == ChangeStatus::Pending).collect()
    }
}
