//! Realm management - Automerge documents with task CRUD
//!
//! A realm is a shared space where tasks are synchronized between peers.
//! Each realm is backed by an Automerge document that provides CRDT-based
//! conflict resolution for concurrent edits.

pub mod doc;

pub use doc::RealmDoc;
