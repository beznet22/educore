//! # RBAC typed identifiers
//!
//! Every RBAC aggregate is keyed by a typed identifier of the form
//! `Id { school: SchoolId, value: Uuid }`. Two ids of different
//! aggregate types are not interchangeable: the type system catches
//! cross-aggregate id confusion at compile time.

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::SchoolId;

macro_rules! rbac_typed_id {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident;
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        $vis struct $name {
            /// The owning school (tenant anchor).
            pub school_id: SchoolId,
            /// The local id (UUIDv7).
            pub value: Uuid,
        }

        impl $name {
            /// Constructs a new typed id from its parts.
            #[must_use]
            pub const fn new(school_id: SchoolId, value: Uuid) -> Self {
                Self { school_id, value }
            }

            /// Returns the local UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.value
            }

            /// Returns the owning school id.
            #[must_use]
            pub const fn school_id(&self) -> SchoolId {
                self.school_id
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}/{}", self.school_id, self.value)
            }
        }
    };
}

rbac_typed_id! {
    /// A typed id for a [`Role`](crate::aggregate::Role) row.
    pub struct RoleId;
}

rbac_typed_id! {
    /// A typed id for a [`Permission`](crate::aggregate::Permission) row.
    pub struct PermissionId;
}

rbac_typed_id! {
    /// A typed id for a [`PermissionSection`](crate::aggregate::PermissionSection) row.
    pub struct PermissionSectionId;
}

rbac_typed_id! {
    /// A typed id for an [`AssignPermission`](crate::entities::AssignPermission) row.
    pub struct AssignPermissionId;
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use educore_core::ids::Identifier;

    #[test]
    fn role_id_constructs_and_displays() {
        let school = SchoolId::from_uuid(Uuid::now_v7());
        let value = Uuid::now_v7();
        let id = RoleId::new(school, value);
        assert_eq!(id.school_id(), school);
        assert_eq!(id.as_uuid(), value);
        assert!(id.to_string().contains(&value.to_string()));
    }

    #[test]
    fn distinct_id_types_are_not_interchangeable() {
        let school = SchoolId::from_uuid(Uuid::now_v7());
        let value = Uuid::now_v7();
        let role = RoleId::new(school, value);
        let perm = PermissionId::new(school, value);
        // same value, different types — `==` is type-scoped
        assert_eq!(role, RoleId::new(school, value));
        assert_ne!(
            format!("{role:?}"),
            format!("{perm:?}"),
            "different types should not compare equal"
        );
    }
}
