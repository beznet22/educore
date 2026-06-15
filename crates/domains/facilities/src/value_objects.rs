//! # Facilities value objects
//!
//! The typed ids (every aggregate is keyed by one), the validated
//! value objects, and the closed enums the facilities aggregates
//! depend on. Per `docs/specs/facilities/value-objects.md`:
//!
//! - Every id is `Id { school_id, value }` — a typed wrapper that
//!   carries the school anchor so the type system catches
//!   cross-tenant confusion at compile time.
//! - Money is `MinorUnits` (i64) per the build-plan § "Risks".
//! - Foreign-key typed ids (`StudentId`, `StaffId`, `RoleId`,
//!   `AcademicYearId`, `ClassId`, `SectionId`) are **re-exported**
//!   from [`educore_academic`](::educore_academic) and
//!   [`educore_hr`](::educore_hr); the facilities crate owns only
//!   the facilities-specific ids.

#![allow(missing_docs)]
#![allow(unused_imports)]

use std::fmt;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;

pub use educore_academic::AcademicYearId;
pub use educore_academic::ClassId;
pub use educore_academic::SectionId;
pub use educore_academic::StudentId;
pub use educore_academic::StudentRecordId;
pub use educore_academic::SubjectId;
pub use educore_hr::value_objects::RoleId;
pub use educore_hr::value_objects::StaffId;

// =============================================================================
// Macro: typed facilities id
// =============================================================================

/// Macro to define the per-aggregate typed id wrapper. Every
/// facilities id follows the same shape: a `school_id` anchor plus
/// a local `Uuid`. The wrapper implements
/// [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], [`Hash`], and
/// the `Display` format `"{school_id}/{value}"`.
macro_rules! facilities_typed_id {
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

// =============================================================================
// Typed ids: 19 aggregate roots + 4 child ids
// =============================================================================

facilities_typed_id! {
    /// A typed id for a [`Vehicle`](crate::aggregate::Vehicle).
    pub struct VehicleId;
}
facilities_typed_id! {
    /// A typed id for a [`Route`](crate::aggregate::Route).
    pub struct RouteId;
}
facilities_typed_id! {
    /// A typed id for an [`AssignVehicle`](crate::aggregate::AssignVehicle).
    pub struct AssignVehicleId;
}
facilities_typed_id! {
    /// A typed id for a [`Dormitory`](crate::aggregate::Dormitory).
    pub struct DormitoryId;
}
facilities_typed_id! {
    /// A typed id for a [`Room`](crate::aggregate::Room).
    pub struct RoomId;
}
facilities_typed_id! {
    /// A typed id for a [`RoomType`](crate::aggregate::RoomType).
    pub struct RoomTypeId;
}
facilities_typed_id! {
    /// A typed id for an [`ItemCategory`](crate::aggregate::ItemCategory).
    pub struct ItemCategoryId;
}
facilities_typed_id! {
    /// A typed id for an [`Item`](crate::aggregate::Item).
    pub struct ItemId;
}
facilities_typed_id! {
    /// A typed id for an [`ItemStore`](crate::aggregate::ItemStore).
    pub struct ItemStoreId;
}
facilities_typed_id! {
    /// A typed id for an [`ItemIssue`](crate::aggregate::ItemIssue).
    pub struct ItemIssueId;
}
facilities_typed_id! {
    /// A typed id for an [`ItemReceive`](crate::aggregate::ItemReceive).
    pub struct ItemReceiveId;
}
facilities_typed_id! {
    /// A typed id for an [`ItemReceiveChild`](crate::aggregate::ItemReceiveChild).
    pub struct ItemReceiveChildId;
}
facilities_typed_id! {
    /// A typed id for an [`ItemSell`](crate::aggregate::ItemSell).
    pub struct ItemSellId;
}
facilities_typed_id! {
    /// A typed id for an [`ItemSellChild`](crate::aggregate::ItemSellChild).
    pub struct ItemSellChildId;
}
facilities_typed_id! {
    /// A typed id for a [`Supplier`](crate::aggregate::Supplier).
    pub struct SupplierId;
}
facilities_typed_id! {
    /// A typed id for a [`RouteStop`](crate::entities::RouteStop) child row.
    pub struct RouteStopId;
}
facilities_typed_id! {
    /// A typed id for a [`RoomAssignment`](crate::entities::RoomAssignment) child row.
    pub struct RoomAssignmentId;
}
facilities_typed_id! {
    /// A typed id for a [`TransportMembership`](crate::entities::TransportMembership) child row.
    pub struct TransportMembershipId;
}
facilities_typed_id! {
    /// A typed id for a [`StoreStocktake`](crate::entities::StoreStocktake) child row.
    pub struct StoreStocktakeId;
}

// =============================================================================
// Names & numbers (validated at construction)
// =============================================================================

/// A validated, non-empty vehicle number. 1..=50 chars,
/// alphanumeric + dash, unique within a school.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VehicleNumber(String);

impl VehicleNumber {
    /// Maximum length of a vehicle number.
    pub const MAX_LEN: usize = 50;

    /// Constructs a `VehicleNumber`, rejecting empty, overlong,
    /// or non-alphanumeric+dash input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_vehicle_number(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for VehicleNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for VehicleNumber {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

fn validate_vehicle_number(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation("vehicle number must not be empty"));
    }
    if s.chars().count() > VehicleNumber::MAX_LEN {
        return Err(DomainError::validation(format!(
            "vehicle number must be at most {} chars, got {}",
            VehicleNumber::MAX_LEN,
            s.chars().count()
        )));
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(DomainError::validation(
            "vehicle number must be alphanumeric + dash + underscore",
        ));
    }
    Ok(())
}

/// A validated vehicle model. 1..=255 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VehicleModel(String);

impl VehicleModel {
    /// Maximum length of a vehicle model.
    pub const MAX_LEN: usize = 255;

    /// Constructs a `VehicleModel`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_vehicle_model(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for VehicleModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_vehicle_model(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation("vehicle model must not be empty"));
    }
    if s.chars().count() > VehicleModel::MAX_LEN {
        return Err(DomainError::validation(format!(
            "vehicle model must be at most {} chars",
            VehicleModel::MAX_LEN
        )));
    }
    Ok(())
}

/// A validated route name. 1..=200 chars, unique within a school-year.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RouteName(String);

impl RouteName {
    /// Maximum length of a route name.
    pub const MAX_LEN: usize = 200;

    /// Constructs a `RouteName`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_route_name(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RouteName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_route_name(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation("route name must not be empty"));
    }
    if s.chars().count() > RouteName::MAX_LEN {
        return Err(DomainError::validation(format!(
            "route name must be at most {} chars",
            RouteName::MAX_LEN
        )));
    }
    Ok(())
}

/// A validated stop name. 1..=200 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StopName(String);

impl StopName {
    /// Maximum length of a stop name.
    pub const MAX_LEN: usize = 200;

    /// Constructs a `StopName`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_stop_name(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StopName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_stop_name(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation("stop name must not be empty"));
    }
    if s.chars().count() > StopName::MAX_LEN {
        return Err(DomainError::validation(format!(
            "stop name must be at most {} chars",
            StopName::MAX_LEN
        )));
    }
    Ok(())
}

/// A validated dormitory name. 1..=200 chars, unique within a school-year.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DormitoryName(String);

impl DormitoryName {
    /// Maximum length of a dormitory name.
    pub const MAX_LEN: usize = 200;

    /// Constructs a `DormitoryName`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_dormitory_name(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DormitoryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_dormitory_name(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation("dormitory name must not be empty"));
    }
    if s.chars().count() > DormitoryName::MAX_LEN {
        return Err(DomainError::validation(format!(
            "dormitory name must be at most {} chars",
            DormitoryName::MAX_LEN
        )));
    }
    Ok(())
}

/// A validated room number. 1..=50 chars, unique within a dormitory.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RoomNumber(String);

impl RoomNumber {
    /// Maximum length of a room number.
    pub const MAX_LEN: usize = 50;

    /// Constructs a `RoomNumber`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_room_number(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RoomNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_room_number(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation("room number must not be empty"));
    }
    if s.chars().count() > RoomNumber::MAX_LEN {
        return Err(DomainError::validation(format!(
            "room number must be at most {} chars",
            RoomNumber::MAX_LEN
        )));
    }
    Ok(())
}

/// A validated room-type name. 1..=255 chars, unique within a school.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RoomTypeName(String);

impl RoomTypeName {
    /// Maximum length of a room-type name.
    pub const MAX_LEN: usize = 255;

    /// Constructs a `RoomTypeName`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_room_type_name(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RoomTypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_room_type_name(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation("room-type name must not be empty"));
    }
    if s.chars().count() > RoomTypeName::MAX_LEN {
        return Err(DomainError::validation(format!(
            "room-type name must be at most {} chars",
            RoomTypeName::MAX_LEN
        )));
    }
    Ok(())
}

/// A validated item-category name. 1..=100 chars, unique within a school.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CategoryName(String);

impl CategoryName {
    /// Maximum length of a category name.
    pub const MAX_LEN: usize = 100;

    /// Constructs a `CategoryName`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_category_name(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CategoryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_category_name(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation("category name must not be empty"));
    }
    if s.chars().count() > CategoryName::MAX_LEN {
        return Err(DomainError::validation(format!(
            "category name must be at most {} chars",
            CategoryName::MAX_LEN
        )));
    }
    Ok(())
}

/// A validated item name. 1..=100 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemName(String);

impl ItemName {
    /// Maximum length of an item name.
    pub const MAX_LEN: usize = 100;

    /// Constructs an `ItemName`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_item_name(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ItemName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_item_name(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation("item name must not be empty"));
    }
    if s.chars().count() > ItemName::MAX_LEN {
        return Err(DomainError::validation(format!(
            "item name must be at most {} chars",
            ItemName::MAX_LEN
        )));
    }
    Ok(())
}

/// A validated item SKU. 1..=50 chars, alphanumeric, unique within a school.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemSku(String);

impl ItemSku {
    /// Maximum length of an item SKU.
    pub const MAX_LEN: usize = 50;

    /// Constructs an `ItemSku`, rejecting empty, overlong, or
    /// non-alphanumeric input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_item_sku(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ItemSku {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_item_sku(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation("item SKU must not be empty"));
    }
    if s.chars().count() > ItemSku::MAX_LEN {
        return Err(DomainError::validation(format!(
            "item SKU must be at most {} chars",
            ItemSku::MAX_LEN
        )));
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(DomainError::validation(
            "item SKU must be alphanumeric + dash + underscore",
        ));
    }
    Ok(())
}

/// A validated store name. 1..=100 chars, unique within a school.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StoreName(String);

impl StoreName {
    /// Maximum length of a store name.
    pub const MAX_LEN: usize = 100;

    /// Constructs a `StoreName`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_store_name(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StoreName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_store_name(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation("store name must not be empty"));
    }
    if s.chars().count() > StoreName::MAX_LEN {
        return Err(DomainError::validation(format!(
            "store name must be at most {} chars",
            StoreName::MAX_LEN
        )));
    }
    Ok(())
}

/// A validated store number. 1..=100 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StoreNumber(String);

impl StoreNumber {
    /// Maximum length of a store number.
    pub const MAX_LEN: usize = 100;

    /// Constructs a `StoreNumber`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_store_number(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StoreNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_store_number(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation("store number must not be empty"));
    }
    if s.chars().count() > StoreNumber::MAX_LEN {
        return Err(DomainError::validation(format!(
            "store number must be at most {} chars",
            StoreNumber::MAX_LEN
        )));
    }
    Ok(())
}

/// A validated supplier name. 1..=100 chars, unique within a school.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SupplierName(String);

impl SupplierName {
    /// Maximum length of a supplier name.
    pub const MAX_LEN: usize = 100;

    /// Constructs a `SupplierName`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_supplier_name(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SupplierName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_supplier_name(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation("supplier name must not be empty"));
    }
    if s.chars().count() > SupplierName::MAX_LEN {
        return Err(DomainError::validation(format!(
            "supplier name must be at most {} chars",
            SupplierName::MAX_LEN
        )));
    }
    Ok(())
}

/// A validated contact-person name. 1..=191 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContactPersonName(String);

impl ContactPersonName {
    /// Maximum length of a contact-person name.
    pub const MAX_LEN: usize = 191;

    /// Constructs a `ContactPersonName`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_contact_person_name(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContactPersonName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_contact_person_name(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation(
            "contact-person name must not be empty",
        ));
    }
    if s.chars().count() > ContactPersonName::MAX_LEN {
        return Err(DomainError::validation(format!(
            "contact-person name must be at most {} chars",
            ContactPersonName::MAX_LEN
        )));
    }
    Ok(())
}

/// A validated reference number. 1..=191 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ReferenceNumber(String);

impl ReferenceNumber {
    /// Maximum length of a reference number.
    pub const MAX_LEN: usize = 191;

    /// Constructs a `ReferenceNumber`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_reference_number(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ReferenceNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_reference_number(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation(
            "reference number must not be empty",
        ));
    }
    if s.chars().count() > ReferenceNumber::MAX_LEN {
        return Err(DomainError::validation(format!(
            "reference number must be at most {} chars",
            ReferenceNumber::MAX_LEN
        )));
    }
    Ok(())
}

// =============================================================================
// Quantities & money (i64 minor units per the build-plan § "Risks")
// =============================================================================

/// A non-negative item quantity in minor units. Used for all
/// receive/issue/sell movements.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemQuantity(pub i64);

impl ItemQuantity {
    /// The zero quantity.
    pub const ZERO: ItemQuantity = ItemQuantity(0);

    /// Constructs a quantity, rejecting negative input.
    pub fn new(v: i64) -> Result<Self> {
        if v < 0 {
            return Err(DomainError::validation(format!(
                "item quantity must be non-negative, got {v}"
            )));
        }
        Ok(Self(v))
    }

    /// Returns the inner value.
    #[must_use]
    pub const fn value(self) -> i64 {
        self.0
    }

    /// Returns `true` if this is the zero quantity.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for ItemQuantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A non-negative unit price in minor units.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnitPrice(pub i64);

impl UnitPrice {
    /// Constructs a unit price, rejecting negative input.
    pub fn new(v: i64) -> Result<Self> {
        if v < 0 {
            return Err(DomainError::validation(format!(
                "unit price must be non-negative, got {v}"
            )));
        }
        Ok(Self(v))
    }

    /// Returns the inner value.
    #[must_use]
    pub const fn value(self) -> i64 {
        self.0
    }
}

impl fmt::Display for UnitPrice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A non-negative sell price in minor units.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SellPrice(pub i64);

impl SellPrice {
    /// Constructs a sell price, rejecting negative input.
    pub fn new(v: i64) -> Result<Self> {
        if v < 0 {
            return Err(DomainError::validation(format!(
                "sell price must be non-negative, got {v}"
            )));
        }
        Ok(Self(v))
    }

    /// Returns the inner value.
    #[must_use]
    pub const fn value(self) -> i64 {
        self.0
    }
}

impl fmt::Display for SellPrice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A non-negative cost-per-bed value in minor units.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CostPerBed(pub i64);

impl CostPerBed {
    /// Constructs a cost-per-bed, rejecting negative input.
    pub fn new(v: i64) -> Result<Self> {
        if v < 0 {
            return Err(DomainError::validation(format!(
                "cost per bed must be non-negative, got {v}"
            )));
        }
        Ok(Self(v))
    }

    /// Returns the inner value.
    #[must_use]
    pub const fn value(self) -> i64 {
        self.0
    }
}

impl fmt::Display for CostPerBed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A non-negative transport fare in minor units.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Fare(pub i64);

impl Fare {
    /// Constructs a fare, rejecting negative input.
    pub fn new(v: i64) -> Result<Self> {
        if v < 0 {
            return Err(DomainError::validation(format!(
                "fare must be non-negative, got {v}"
            )));
        }
        Ok(Self(v))
    }

    /// Returns the inner value.
    #[must_use]
    pub const fn value(self) -> i64 {
        self.0
    }
}

impl fmt::Display for Fare {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A non-negative distance in kilometres.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Distance(pub i64);

impl Distance {
    /// Constructs a distance, rejecting negative input.
    pub fn new(v: i64) -> Result<Self> {
        if v < 0 {
            return Err(DomainError::validation(format!(
                "distance must be non-negative, got {v}"
            )));
        }
        Ok(Self(v))
    }

    /// Returns the inner value.
    #[must_use]
    pub const fn value(self) -> i64 {
        self.0
    }
}

impl fmt::Display for Distance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A non-negative stock-on-hand value in minor units. Persists on
/// the `Item` aggregate and is the source of truth for the
/// inventory conservation invariant.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StockOnHand(pub i64);

impl StockOnHand {
    /// The zero stock value.
    pub const ZERO: StockOnHand = StockOnHand(0);

    /// Constructs a stock value, rejecting negative input.
    pub fn new(v: i64) -> Result<Self> {
        if v < 0 {
            return Err(DomainError::validation(format!(
                "stock on hand must be non-negative, got {v}"
            )));
        }
        Ok(Self(v))
    }

    /// Returns the inner value.
    #[must_use]
    pub const fn value(self) -> i64 {
        self.0
    }

    /// Returns `true` if this is the zero stock value.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for StockOnHand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A positive dormitory intake. Capacity in students per year.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Intake(pub u32);

impl Intake {
    /// Constructs an intake, rejecting zero.
    pub fn new(v: u32) -> Result<Self> {
        if v == 0 {
            return Err(DomainError::validation("dormitory intake must be positive"));
        }
        Ok(Self(v))
    }

    /// Returns the inner value.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

impl fmt::Display for Intake {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A positive number of beds in a room.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NumberOfBed(pub u32);

impl NumberOfBed {
    /// Constructs a number-of-bed, rejecting zero.
    pub fn new(v: u32) -> Result<Self> {
        if v == 0 {
            return Err(DomainError::validation("number of beds must be positive"));
        }
        Ok(Self(v))
    }

    /// Returns the inner value.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

impl fmt::Display for NumberOfBed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A 1-indexed bed number within a room.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BedNumber(pub u32);

impl BedNumber {
    /// Constructs a bed number, rejecting zero.
    pub fn new(v: u32) -> Result<Self> {
        if v == 0 {
            return Err(DomainError::validation("bed number must be positive"));
        }
        Ok(Self(v))
    }

    /// Returns the inner value.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

impl fmt::Display for BedNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The model year of a vehicle. 1950..=current calendar year.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MadeYear(pub i32);

impl MadeYear {
    /// Constructs a `MadeYear`, rejecting out-of-range values.
    pub fn new(v: i32, current_year: i32) -> Result<Self> {
        if !(1950..=current_year).contains(&v) {
            return Err(DomainError::validation(format!(
                "made year must be in 1950..={current_year}, got {v}"
            )));
        }
        Ok(Self(v))
    }

    /// Returns the inner value.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl fmt::Display for MadeYear {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// =============================================================================
// Status enums
// =============================================================================

/// The gender scope of a dormitory. Per
/// `docs/specs/facilities/aggregates.md#dormitory`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DormitoryType {
    /// A boys' dormitory.
    #[default]
    Boys,
    /// A girls' dormitory.
    Girls,
}

impl DormitoryType {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Boys => "boys",
            Self::Girls => "girls",
        }
    }

    /// Parses a snake_case wire string.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "boys" => Ok(Self::Boys),
            "girls" => Ok(Self::Girls),
            other => Err(DomainError::validation(format!(
                "unknown dormitory type: {other:?}"
            ))),
        }
    }

    /// Returns the single-character storage code per
    /// `docs/specs/facilities/tables.md`.
    #[must_use]
    pub const fn as_db_code(self) -> &'static str {
        match self {
            Self::Boys => "B",
            Self::Girls => "G",
        }
    }
}

impl fmt::Display for DormitoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The lifecycle status of an `ItemIssue` row. The state machine
/// is Issued → (Returned | PartiallyReturned | Lost).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueStatus {
    /// The item has been issued to the recipient.
    #[default]
    Issued,
    /// The item has been fully returned.
    Returned,
    /// The item has been partially returned (some quantity still
    /// outstanding).
    PartiallyReturned,
    /// The item has been reported lost.
    Lost,
}

impl IssueStatus {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Issued => "issued",
            Self::Returned => "returned",
            Self::PartiallyReturned => "partially_returned",
            Self::Lost => "lost",
        }
    }

    /// Parses a snake_case wire string.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "issued" => Ok(Self::Issued),
            "returned" => Ok(Self::Returned),
            "partially_returned" => Ok(Self::PartiallyReturned),
            "lost" => Ok(Self::Lost),
            other => Err(DomainError::validation(format!(
                "unknown issue status: {other:?}"
            ))),
        }
    }
}

impl fmt::Display for IssueStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The payment status of a `ItemReceive` or `ItemSell` row.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PaidStatus {
    /// Fully paid (`total_paid == grand_total`).
    #[default]
    Paid,
    /// Partially paid (`0 < total_paid < grand_total`).
    Partial,
    /// Unpaid (`total_paid == 0`).
    Unpaid,
    /// Refunded (the sale was reversed via `RefundItemSell`).
    Refunded,
}

impl PaidStatus {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Paid => "paid",
            Self::Partial => "partial",
            Self::Unpaid => "unpaid",
            Self::Refunded => "refunded",
        }
    }

    /// Parses a snake_case wire string.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "paid" => Ok(Self::Paid),
            "partial" => Ok(Self::Partial),
            "unpaid" => Ok(Self::Unpaid),
            "refunded" => Ok(Self::Refunded),
            other => Err(DomainError::validation(format!(
                "unknown paid status: {other:?}"
            ))),
        }
    }
}

impl fmt::Display for PaidStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The payment method used to settle a receive or sell.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PaymentMethod {
    /// Cash payment.
    #[default]
    Cash,
    /// Bank transfer.
    Bank,
    /// Cheque.
    Cheque,
    /// Card payment.
    Card,
    /// Wallet payment.
    Wallet,
}

impl PaymentMethod {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cash => "cash",
            Self::Bank => "bank",
            Self::Cheque => "cheque",
            Self::Card => "card",
            Self::Wallet => "wallet",
        }
    }

    /// Parses a snake_case wire string.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "cash" => Ok(Self::Cash),
            "bank" => Ok(Self::Bank),
            "cheque" => Ok(Self::Cheque),
            "card" => Ok(Self::Card),
            "wallet" => Ok(Self::Wallet),
            other => Err(DomainError::validation(format!(
                "unknown payment method: {other:?}"
            ))),
        }
    }
}

impl fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The lifecycle status of a `Supplier` row.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupplierStatus {
    /// The supplier is active and may receive new purchase orders.
    #[default]
    Active,
    /// The supplier is inactive (no new POs; historical receives
    /// remain valid).
    Inactive,
    /// The supplier is blacklisted (no new POs; all historical
    /// receives flagged in audit).
    Blacklisted,
}

impl SupplierStatus {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Blacklisted => "blacklisted",
        }
    }

    /// Parses a snake_case wire string.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            "blacklisted" => Ok(Self::Blacklisted),
            other => Err(DomainError::validation(format!(
                "unknown supplier status: {other:?}"
            ))),
        }
    }
}

impl fmt::Display for SupplierStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The operational status of a `Vehicle` row. Per
/// `docs/specs/facilities/value-objects.md` § Status Enums.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VehicleStatus {
    /// The vehicle is active and may be assigned to a route.
    #[default]
    Active,
    /// The vehicle is in maintenance (no new assignments; current
    /// assignment may continue).
    Maintenance,
    /// The vehicle is retired (no new assignments; historical
    /// receives remain valid).
    Retired,
}

impl VehicleStatus {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Maintenance => "maintenance",
            Self::Retired => "retired",
        }
    }

    /// Parses a snake_case wire string.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "active" => Ok(Self::Active),
            "maintenance" => Ok(Self::Maintenance),
            "retired" => Ok(Self::Retired),
            other => Err(DomainError::validation(format!(
                "unknown vehicle status: {other:?}"
            ))),
        }
    }
}

impl fmt::Display for VehicleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A simple active/inactive flag carried on every persisted row.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActiveStatus {
    /// The row is active.
    #[default]
    Active,
    /// The row is inactive.
    Inactive,
}

impl ActiveStatus {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
        }
    }

    /// Returns `true` if the row is active.
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Active)
    }
}

impl fmt::Display for ActiveStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// =============================================================================
// Identity
// =============================================================================

/// A validated E.164-style phone number. 7..=20 digits,
/// optionally prefixed with `+`. Per
/// `docs/specs/facilities/value-objects.md` § Identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    /// Maximum length of a phone number.
    pub const MAX_LEN: usize = 20;

    /// Constructs a `PhoneNumber`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_phone_number(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_phone_number(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation("phone number must not be empty"));
    }
    if s.len() > PhoneNumber::MAX_LEN {
        return Err(DomainError::validation(format!(
            "phone number must be at most {} chars",
            PhoneNumber::MAX_LEN
        )));
    }
    let trimmed = s.strip_prefix('+').unwrap_or(s);
    if !trimmed.chars().all(|c| c.is_ascii_digit()) {
        return Err(DomainError::validation(
            "phone number must be E.164 (+ prefix optional, digits only)",
        ));
    }
    Ok(())
}

/// A validated RFC 5322-style email address. 1..=200 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EmailAddress(String);

impl EmailAddress {
    /// Maximum length of an email address.
    pub const MAX_LEN: usize = 200;

    /// Constructs an `EmailAddress`, rejecting empty or overlong
    /// input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_email_address(&s)?;
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_email_address(s: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation("email address must not be empty"));
    }
    if s.len() > EmailAddress::MAX_LEN {
        return Err(DomainError::validation(format!(
            "email address must be at most {} chars",
            EmailAddress::MAX_LEN
        )));
    }
    if !s.contains('@') || s.starts_with('@') || s.ends_with('@') {
        return Err(DomainError::validation(
            "email address must contain exactly one '@' separator",
        ));
    }
    Ok(())
}

/// A validated address string. 1..=500 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Address(String);

impl Address {
    /// Maximum length of an address.
    pub const MAX_LEN: usize = 500;

    /// Constructs an `Address`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation("address must not be empty"));
        }
        if s.len() > Address::MAX_LEN {
            return Err(DomainError::validation(format!(
                "address must be at most {} chars",
                Address::MAX_LEN
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A free-text description. 0..=500 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct Description(String);

impl Description {
    /// Maximum length of a description.
    pub const MAX_LEN: usize = 500;

    /// Constructs a `Description`, rejecting overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.len() > Description::MAX_LEN {
            return Err(DomainError::validation(format!(
                "description must be at most {} chars",
                Description::MAX_LEN
            )));
        }
        Ok(Self(s))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Description {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A free-text note. 0..=500 chars.
pub type Note = Description;

// =============================================================================
// Specification helpers
// =============================================================================

/// The recipient of an `ItemIssue`. The recipient may be a staff
/// member, a student, or a role (a generic "any member of role X").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id")]
pub enum IssueRecipient {
    /// The issue is to a staff member.
    Staff(StaffId),
    /// The issue is to a student.
    Student(StudentId),
    /// The issue is to any member of a role.
    Role(RoleId),
}

impl IssueRecipient {
    /// Returns the human-readable wire form.
    #[must_use]
    pub const fn kind(self) -> &'static str {
        match self {
            Self::Staff(_) => "staff",
            Self::Student(_) => "student",
            Self::Role(_) => "role",
        }
    }
}

impl fmt::Display for IssueRecipient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Staff(id) => write!(f, "staff:{id}"),
            Self::Student(id) => write!(f, "student:{id}"),
            Self::Role(id) => write!(f, "role:{id}"),
        }
    }
}

/// A spec for a single receive line in a `ReceiveItem` command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemReceiveLineSpec {
    /// The item being received.
    pub item_id: ItemId,
    /// The unit price per unit received.
    pub unit_price: UnitPrice,
    /// The quantity received.
    pub quantity: ItemQuantity,
    /// An optional description.
    pub description: Option<Description>,
}

impl ItemReceiveLineSpec {
    /// Computes the line subtotal as `unit_price * quantity`.
    #[must_use]
    pub fn sub_total(&self) -> i64 {
        self.unit_price
            .value()
            .saturating_mul(self.quantity.value())
    }
}

/// A spec for a single sell line in a `SellItem` command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemSellLineSpec {
    /// The item being sold.
    pub item_id: ItemId,
    /// The sell price per unit.
    pub sell_price: SellPrice,
    /// The quantity sold.
    pub quantity: ItemQuantity,
    /// An optional description.
    pub description: Option<Description>,
}

impl ItemSellLineSpec {
    /// Computes the line subtotal as `sell_price * quantity`.
    #[must_use]
    pub fn sub_total(&self) -> i64 {
        self.sell_price
            .value()
            .saturating_mul(self.quantity.value())
    }
}

/// The payload of a single `RouteStop` event. Per
/// `docs/specs/facilities/events.md`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteStopSpec {
    /// The stop's order within the route.
    pub stop_order: u32,
    /// The stop's name.
    pub stop_name: StopName,
    /// An optional pickup time (a `HH:MM` 24h string).
    pub pickup_time: Option<NaiveTime>,
    /// An optional fare override (the default is the route's fare).
    pub fare_override: Option<Fare>,
}

/// NaiveTime re-export for the [`RouteStopSpec`].
pub use chrono::NaiveTime;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;

    #[test]
    fn vehicle_number_rejects_empty() {
        let err = VehicleNumber::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn vehicle_number_accepts_alphanumeric() {
        let n = VehicleNumber::new("GJ-05-AB-1234").unwrap();
        assert_eq!(n.as_str(), "GJ-05-AB-1234");
    }

    #[test]
    fn vehicle_number_rejects_oversized() {
        let s = "a".repeat(VehicleNumber::MAX_LEN + 1);
        let err = VehicleNumber::new(s).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn item_quantity_rejects_negative() {
        let err = ItemQuantity::new(-1).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn item_quantity_zero() {
        assert!(ItemQuantity::ZERO.is_zero());
        assert_eq!(ItemQuantity::ZERO.value(), 0);
    }

    #[test]
    fn made_year_enforces_range() {
        let err = MadeYear::new(1900, 2026).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
        let err = MadeYear::new(2030, 2026).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
        let ok = MadeYear::new(2020, 2026).unwrap();
        assert_eq!(ok.value(), 2020);
    }

    #[test]
    fn dormitory_type_round_trip() {
        for t in [DormitoryType::Boys, DormitoryType::Girls] {
            assert_eq!(DormitoryType::parse(t.as_str()).unwrap(), t);
            assert_eq!(
                t.as_db_code(),
                if t == DormitoryType::Boys { "B" } else { "G" }
            );
        }
    }

    #[test]
    fn issue_status_round_trip() {
        for s in [
            IssueStatus::Issued,
            IssueStatus::Returned,
            IssueStatus::PartiallyReturned,
            IssueStatus::Lost,
        ] {
            assert_eq!(IssueStatus::parse(s.as_str()).unwrap(), s);
        }
    }

    #[test]
    fn paid_status_round_trip() {
        for s in [
            PaidStatus::Paid,
            PaidStatus::Partial,
            PaidStatus::Unpaid,
            PaidStatus::Refunded,
        ] {
            assert_eq!(PaidStatus::parse(s.as_str()).unwrap(), s);
        }
    }

    #[test]
    fn payment_method_round_trip() {
        for m in [
            PaymentMethod::Cash,
            PaymentMethod::Bank,
            PaymentMethod::Cheque,
            PaymentMethod::Card,
            PaymentMethod::Wallet,
        ] {
            assert_eq!(PaymentMethod::parse(m.as_str()).unwrap(), m);
        }
    }

    #[test]
    fn phone_number_rejects_letters() {
        let err = PhoneNumber::new("+1-800-ABC").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn email_address_rejects_no_at_sign() {
        let err = EmailAddress::new("not-an-email").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn typed_ids_carry_their_school_anchor() {
        let school_a = SchoolId(Uuid::now_v7());
        let school_b = SchoolId(Uuid::now_v7());
        let value = Uuid::now_v7();
        let id = ItemId::new(school_a, value);
        assert_eq!(id.school_id(), school_a);
        assert_ne!(id.school_id(), school_b);
    }

    #[test]
    fn typed_ids_display_format() {
        let school = SchoolId(Uuid::now_v7());
        let value = Uuid::now_v7();
        let id = ItemId::new(school, value);
        let s = id.to_string();
        assert!(s.contains('/'));
        assert!(s.starts_with(&school.to_string()));
    }

    #[test]
    fn line_specs_compute_subtotal() {
        let item_id = ItemId::new(SchoolId(Uuid::now_v7()), Uuid::now_v7());
        let line = ItemReceiveLineSpec {
            item_id,
            unit_price: UnitPrice(50),
            quantity: ItemQuantity(10),
            description: None,
        };
        assert_eq!(line.sub_total(), 500);
    }
}
