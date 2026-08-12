//! Scope system using u128 bit-packing taken from syntect
//!
//! Scopes like "source.rust.meta.function" are packed into a single u128:
//! Memory layout: `[atom0][atom1][atom2][atom3][atom4][atom5][atom6][atom7]`
//! Each atom is 16 bits, storing repository_index + 1 (0 = unused slot)
//! Any atom above the 8th will be ignored

use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};

pub const MAX_ATOMS_IN_SCOPE: usize = 8;
// Leaving room for 0 and MAX
pub const MAX_ATOMS_IN_REPOSITORY: usize = u16::MAX as usize - 2;
// Repository index for empty atoms
pub const EMPTY_ATOM_INDEX: usize = u16::MAX as usize - 1;
// Stored atom number for empty atoms
pub const EMPTY_ATOM_NUMBER: u16 = u16::MAX;

/// A scope represents a hierarchical position in source code like "source.rust.meta.function"
/// Internally stored as a single u128 with up to 8 atoms packed as 16-bit indices
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Default, Hash, Serialize, Deserialize)]
pub struct Scope {
    /// Packed atoms in MSB-first order for lexicographic comparison
    atoms: u128,
}

impl Scope {
    /// Create a new scope from a dot-separated string, truncating to 8 atoms if longer.
    /// It returns a Vec as the scope string might contain spaces, in which case it will split
    /// on it and return multiple scopes
    /// e.g., "string.json support.type.property-name.json" -> [Scope("string.json"), Scope("support.type.property-name.json")]
    pub fn new(scope_str: &str) -> Vec<Scope> {
        let mut repo = lock_global_scope_repo();
        scope_str
            .split_whitespace()
            .map(|part| repo.parse(part.trim()))
            .collect()
    }

    /// Extract a single atom at the given index (0-7)
    /// Returns atom_number: 0 for unused slots, u16::MAX for empty atoms or (atom_index + 1) for valid
    /// non-empty atoms
    #[inline]
    pub fn atom_at(self, index: usize) -> u16 {
        debug_assert!(index < MAX_ATOMS_IN_SCOPE);
        // MSB-first layout: index 0 is in bits [127:112], index 1 in [111:96], etc.
        let shift = (MAX_ATOMS_IN_SCOPE - 1 - index) * 16;
        ((self.atoms >> shift) & 0xFFFF) as u16
    }

    /// Count the number of atoms in this scope
    #[inline]
    pub fn len(self) -> u32 {
        MAX_ATOMS_IN_SCOPE as u32 - self.missing_atoms()
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.atoms == 0
    }

    /// Count unused slots by finding trailing zeros (LSB side has unused atoms)
    /// Since atoms are packed MSB-first, unused slots create trailing zeros
    #[inline]
    fn missing_atoms(self) -> u32 {
        self.atoms.trailing_zeros() / 16
    }

    /// Check if this scope is a prefix of another scope using bitwise masking
    /// This is the core operation for theme selector matching - must be O(1)
    #[inline]
    pub fn is_prefix_of(self, other: Scope) -> bool {
        let missing = self.missing_atoms();

        if missing == MAX_ATOMS_IN_SCOPE as u32 {
            return true; // Empty scope is prefix of everything
        }

        // Create a mask that covers only the prefix portion
        // For a 2-atom prefix (missing=6), mask covers top 32 bits (2 * 16)
        let mask = u128::MAX << (missing * 16);

        // XOR finds differing bits, mask isolates the prefix we care about
        (self.atoms ^ other.atoms) & mask == 0
    }

    /// Convert back to string form - expensive, only use for debugging/display
    pub fn build_string(self) -> String {
        let repo = lock_global_scope_repo();
        repo.to_string(self)
    }
}

impl fmt::Debug for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Scope(\"{}\")", self.build_string())
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.build_string())
    }
}

/// Global repository that maps atom strings to indices for deduplication
#[derive(Debug, Clone, Default)]
pub(crate) struct ScopeRepository {
    /// Index-to-string mapping
    pub(crate) atoms: Vec<String>,
    /// String-to-index for fast lookup
    atom_index_map: HashMap<String, usize>,
}

impl ScopeRepository {
    fn new() -> Self {
        Self::default()
    }

    #[cfg(feature = "dump")]
    pub fn from_atoms(atoms: Vec<String>) -> Self {
        let mut atom_index_map = HashMap::with_capacity(atoms.len());
        for (index, atom) in atoms.iter().enumerate() {
            atom_index_map.insert(atom.clone(), index);
        }
        Self {
            atoms,
            atom_index_map,
        }
    }

    /// Get existing index or register new atom, returning repository index
    /// Returns atom_index: 0-based position in the repository atoms vector
    fn atom_to_index(&mut self, atom: &str) -> usize {
        // Handle empty atoms specially - return reserved index
        if atom.is_empty() {
            return EMPTY_ATOM_INDEX;
        }

        // Fast path: atom already registered
        if let Some(&index) = self.atom_index_map.get(atom) {
            return index;
        }

        if self.atoms.len() >= MAX_ATOMS_IN_REPOSITORY {
            panic!(
                "Too many atoms in repository: exceeded MAX_ATOMS_IN_REPOSITORY of {MAX_ATOMS_IN_REPOSITORY}"
            );
        }

        // Slow path: register new atom
        let index = self.atoms.len();
        self.atoms.push(atom.to_owned());
        self.atom_index_map.insert(atom.to_owned(), index);
        index
    }

    /// Convert atom number back to string
    /// Takes atom_number: 1-based encoded value stored in Scope bits (atom_index + 1)
    pub(crate) fn atom_number_to_str(&self, atom_number: u16) -> &str {
        debug_assert!(atom_number > 0);
        &self.atoms[(atom_number - 1) as usize]
    }

    /// Parse dot-separated string into bit-packed scope, truncating if > 8 atoms
    fn parse(&mut self, scope_str: &str) -> Scope {
        if scope_str.is_empty() {
            return Scope::default();
        }

        let parts: Vec<&str> = scope_str.split('.').collect();
        let atoms_to_process = parts.len().min(MAX_ATOMS_IN_SCOPE); // Truncate to 8 atoms
        let mut atoms = 0u128;

        for (i, &part) in parts.iter().take(atoms_to_process).enumerate() {
            // Process ALL atoms including empty ones (now handled by atom_to_index)
            let index = self.atom_to_index(part); // atom_index: 0-based repository position
            // Convert to atom_number: 1-based encoded value (index + 1) so that 0 can mean "unused slot"
            let atom_number = (index + 1) as u128;

            // Pack MSB-first: first atom goes in highest bits for lexicographic ordering
            let shift = (MAX_ATOMS_IN_SCOPE - 1 - i) * 16;
            atoms |= atom_number << shift;
        }

        Scope { atoms }
    }

    /// Reconstruct string from bit-packed scope
    fn to_string(&self, scope: Scope) -> String {
        let mut parts = Vec::new();

        for i in 0..MAX_ATOMS_IN_SCOPE {
            match scope.atom_at(i) {
                0 => break,
                a if a == EMPTY_ATOM_NUMBER => {
                    parts.push("");
                }
                a => {
                    parts.push(self.atom_number_to_str(a));
                }
            }
        }

        parts.join(".")
    }
}

/// Global singleton repository with thread-safe access
static SCOPE_REPO: std::sync::OnceLock<Mutex<ScopeRepository>> = std::sync::OnceLock::new();

pub(crate) fn lock_global_scope_repo() -> MutexGuard<'static, ScopeRepository> {
    SCOPE_REPO
        .get_or_init(|| Mutex::new(ScopeRepository::new()))
        .lock()
        .expect("Failed to lock scope repository")
}

/// Replace the global ScopeRepository with a new one
/// This is used when loading a registry from disk to restore the complete state
/// Note: Only the first call succeeds; subsequent calls are silently ignored
#[cfg(feature = "dump")]
pub(crate) fn replace_global_scope_repo(new_repo: ScopeRepository) {
    // Ignore error if already set: first call wins
    let _ = SCOPE_REPO.set(Mutex::new(new_repo));
}

/// Represents a Vec<Scope> as a u32
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Default, Hash, Serialize, Deserialize,
)]
pub(crate) struct ScopeListId(u32);

/// We keep this const here instead of storing it in the interner since we would need to
/// create a dummy scope otherwise
pub(crate) const EMPTY_SCOPE_LIST: ScopeListId = ScopeListId(0);

impl ScopeListId {
    #[inline]
    pub(crate) fn as_index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ScopeListNode {
    parent: ScopeListId,
    scope: Scope,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct ScopeInterner {
    nodes: Vec<ScopeListNode>,
    lookup: HashMap<(ScopeListId, Scope), ScopeListId>,
}

impl ScopeInterner {
    pub(crate) fn num_nodes(&self) -> usize {
        self.nodes.len() + 1
    }

    pub(crate) fn push(&mut self, parent: ScopeListId, scope: Scope) -> ScopeListId {
        *self.lookup.entry((parent, scope)).or_insert_with(|| {
            let node = ScopeListNode { parent, scope };
            self.nodes.push(node);
            ScopeListId(self.nodes.len() as u32)
        })
    }

    pub(crate) fn extend(&mut self, parent: ScopeListId, scopes: &[Scope]) -> ScopeListId {
        let mut current = parent;
        for scope in scopes {
            current = self.push(current, *scope);
        }
        current
    }

    pub(crate) fn get_scopes(&self, id: ScopeListId) -> Vec<Scope> {
        let mut out = vec![];
        let mut current = id;
        while current != EMPTY_SCOPE_LIST {
            // we have EMPTY_SCOPE_LIST so interned ids are 1-based basically
            let node = &self.nodes[current.as_index() - 1];
            out.push(node.scope);
            current = node.parent;
        }
        out.reverse();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_scope_creation() {
        let scope = Scope::new("source.rust.meta.function")[0];
        assert_eq!(scope.len(), 4);
        assert_eq!(scope.build_string(), "source.rust.meta.function");
    }

    #[test]
    fn test_empty_scope() {
        let scope = Scope::new("");
        assert_eq!(scope.len(), 0);
        assert!(scope.is_empty());
    }

    #[test]
    fn test_prefix_matching() {
        let prefix = Scope::new("source.rust")[0];
        let full = Scope::new("source.rust.meta.function")[0];
        let different = Scope::new("source.javascript")[0];

        assert!(prefix.is_prefix_of(full));
        assert!(prefix.is_prefix_of(prefix));
        assert!(!prefix.is_prefix_of(different));
    }

    #[test]
    fn test_atom_truncation() {
        // Scopes with >8 atoms should be truncated to first 8
        // only an issue with cpp grammar
        let long_scope = Scope::new("a.b.c.d.e.f.g.h.i.j.k.l")[0];
        assert_eq!(long_scope.len(), 8);
        assert_eq!(long_scope.build_string(), "a.b.c.d.e.f.g.h");
    }

    #[test]
    fn test_atom_extraction() {
        let scope = Scope::new("source.rust.meta")[0];

        assert_ne!(scope.atom_at(0), 0); // "source" is present
        assert_ne!(scope.atom_at(1), 0); // "rust" is present
        assert_ne!(scope.atom_at(2), 0); // "meta" is present
        assert_eq!(scope.atom_at(3), 0); // unused slot
        assert_eq!(scope.atom_at(7), 0); // unused slot
    }

    #[test]
    fn test_scope_ordering() {
        let scope1 = Scope::new("source.rust")[0];
        let scope2 = Scope::new("source.rust.meta")[0];

        // Longer scopes should sort after shorter prefixes
        assert!(scope1 < scope2);
    }

    #[test]
    fn test_scope_equality() {
        let scope1 = Scope::new("source.rust.meta")[0];
        let scope2 = Scope::new("source.rust.meta")[0];
        let scope3 = Scope::new("source.rust")[0];

        assert_eq!(scope1, scope2);
        assert_ne!(scope1, scope3);
    }

    #[test]
    fn test_empty_atom_preservation() {
        let scope = Scope::new("meta.tag.object.svg..end.html")[0];
        assert_eq!(scope.build_string(), "meta.tag.object.svg..end.html");
        assert_eq!(scope.len(), 7);
    }

    #[test]
    fn test_empty_atoms_various_positions() {
        assert_eq!(Scope::new("a...b")[0].build_string(), "a...b");
        assert_eq!(Scope::new(".start.end")[0].build_string(), ".start.end");
        assert_eq!(Scope::new("start.end.")[0].build_string(), "start.end.");
    }

    #[test]
    fn test_interner() {
        let mut interner = ScopeInterner::default();
        let id1 = interner.push(EMPTY_SCOPE_LIST, Scope::new("source.rust")[0]);
        let id2 = interner.push(id1, Scope::new("comment")[0]);
        // same one as above
        let id3 = interner.push(id1, Scope::new("comment")[0]);
        assert_eq!(id2, id3);
        assert_eq!(interner.nodes.len(), 2);
        let id4 = interner.extend(id2, &[Scope::new("variable")[0], Scope::new("other")[0]]);

        let res = interner.get_scopes(id2);
        assert_eq!(
            res,
            vec![Scope::new("source.rust")[0], Scope::new("comment")[0]]
        );
        let res = interner.get_scopes(id4);
        assert_eq!(
            res,
            vec![
                Scope::new("source.rust")[0],
                Scope::new("comment")[0],
                Scope::new("variable")[0],
                Scope::new("other")[0]
            ]
        );
    }
}
