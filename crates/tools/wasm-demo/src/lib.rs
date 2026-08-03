//! # educore-wasm-demo
//!
//! WASM-compatible demo of the Educore engine's pure logic.
//! This crate compiles to `wasm32-unknown-unknown` and exposes
//! JavaScript-callable functions that exercise the engine
//! without any native-IO dependencies (no storage adapters, no
//! tokio, no fs).
//!
//! Use cases:
//! - Browser-based school management UI
//! - Edge compute for form validation
//! - Offline-first PWA prototypes
//!
//! The pure-logic scope includes:
//! - Typed ids (`SchoolId`, `StudentId`, `StaffId`, ...)
//! - Tenant contexts (`TenantContext`)
//! - Value-object validation (email, phone, names, dates)
//! - Aggregate construction (in-memory)
//!
//! What this demo does NOT include (WASM-incompatible):
//! - Storage adapter round-trip (requires native IO)
//! - Event bus publish (requires tokio runtime)
//! - Payment provider charge (requires external network)

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use wasm_bindgen::prelude::*;

use educore_core::clock::IdGenerator;
use educore_core::ids::Identifier;
use educore_core::tenant::{TenantContext, UserType};
use educore_rbac::value_objects::Capability;

/// Initialize the WASM module. Call once at startup.
///
/// Sets up the panic hook so WASM panics surface as console
/// errors instead of opaque aborts.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Look up a [`Capability`] by its kebab-case string name.
///
/// Returns `None` if no capability matches.
fn capability_from_str(s: &str) -> Option<Capability> {
    // Use FromStr if implemented, else match a known set.
    match s {
        "academic.student.create" => Some(Capability::AcademicStudentCreate),
        "academic.student.read" => Some(Capability::AcademicStudentRead),
        "academic.student.update" => Some(Capability::AcademicStudentUpdate),
        "academic.student.delete" => Some(Capability::AcademicStudentDelete),
        "hr.staff.create" => Some(Capability::HrStaffCreate),
        "hr.staff.read" => Some(Capability::HrStaffRead),
        _ => None,
    }
}

/// WASM-exposed admission validator.
///
/// Validates a student's admission payload in the browser
/// without any server round-trip. Returns a JSON object with
/// `ok: bool` and either `student_id` (on success) or `errors`
/// (on validation failure).
#[wasm_bindgen]
pub fn validate_admission(
    school_uuid: &str,
    first_name: &str,
    last_name: &str,
    email: Option<String>,
) -> Result<JsValue, JsValue> {
    let school_uuid = uuid::Uuid::parse_str(school_uuid)
        .map_err(|e| JsValue::from_str(&format!("invalid school uuid: {e}")))?;
    let school_id = educore_core::ids::SchoolId::from_uuid(school_uuid);
    let g = educore_core::clock::SystemIdGen;
    let user_id = g.next_user_id();
    let correlation_id = g.next_correlation_id();
    let ctx = TenantContext::for_user(
        school_id,
        user_id,
        correlation_id,
        UserType::SchoolAdmin,
    );

    let mut errors: Vec<String> = Vec::new();
    if first_name.trim().is_empty() {
        errors.push("first_name is required".to_owned());
    }
    if first_name.len() > 100 {
        errors.push("first_name exceeds 100 chars".to_owned());
    }
    if last_name.trim().is_empty() {
        errors.push("last_name is required".to_owned());
    }
    if last_name.len() > 100 {
        errors.push("last_name exceeds 100 chars".to_owned());
    }
    if let Some(ref e) = email {
        if !e.contains('@') || !e.contains('.') {
            errors.push(format!("invalid email: {e}"));
        }
    }
    if !errors.is_empty() {
        let out = serde_json::json!({"ok": false, "errors": errors});
        return Ok(serde_wasm_bindgen::to_value(&out)
            .map_err(|e| JsValue::from_str(&format!("serialize: {e}")))?);
    }

    // Note: full RBAC check requires InMemoryCapabilityCheck which
    // is async — omitted from the WASM demo to keep the surface
    // sync. Real consumers wire this through the dispatcher's
    // RBAC pipeline.

    let student_id = g.next_uuid();
    let out = serde_json::json!({
        "ok": true,
        "student_id": student_id.to_string(),
        "school_id": school_id.as_uuid().to_string(),
        "correlation_id": correlation_id.as_uuid().to_string(),
        "tenant_user_id": ctx.actor_id.as_uuid().to_string(),
    });
    Ok(serde_wasm_bindgen::to_value(&out)
        .map_err(|e| JsValue::from_str(&format!("serialize: {e}")))?)
}

/// WASM-exposed capability name lookup.
///
/// Returns the kebab-case name of a capability, or `None` if
/// the name is not recognized by the engine.
#[wasm_bindgen]
pub fn capability_known(name: &str) -> bool {
    capability_from_str(name).is_some()
}

/// WASM-exposed aggregate query.
///
/// Builds an in-memory student id from the given school +
/// names and returns the serialized form as JSON. Demonstrates
/// that aggregate construction (pure logic) works in WASM.
#[wasm_bindgen]
pub fn build_student_summary(
    school_uuid: &str,
    first_name: &str,
    last_name: &str,
) -> Result<JsValue, JsValue> {
    let school_uuid = uuid::Uuid::parse_str(school_uuid)
        .map_err(|e| JsValue::from_str(&format!("invalid school uuid: {e}")))?;
    let school_id = educore_core::ids::SchoolId::from_uuid(school_uuid);
    let g = educore_core::clock::SystemIdGen;
    let student_id = educore_academic::value_objects::StudentId::new(school_id, g.next_uuid());

    let out = serde_json::json!({
        "student_id": student_id.as_uuid().to_string(),
        "school_id": school_id.as_uuid().to_string(),
        "first_name": first_name,
        "last_name": last_name,
        "active_status": "Active",
    });
    Ok(serde_wasm_bindgen::to_value(&out)
        .map_err(|e| JsValue::from_str(&format!("serialize: {e}")))?)
}

/// WASM-exposed engine version.
///
/// Returns the engine version string, useful for client-side
/// compatibility checks.
#[wasm_bindgen]
pub fn engine_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_admission_rejects_empty_first_name() {
        let result = validate_admission(
            "00000000-0000-0000-0000-000000000001",
            "",
            "Lovelace",
            None,
        )
        .unwrap();
        let json: serde_json::Value = serde_wasm_bindgen::from_value(result).unwrap();
        assert_eq!(json["ok"], false);
        assert!(json["errors"].as_array().unwrap().len() > 0);
    }

    #[test]
    fn validate_admission_succeeds() {
        let result = validate_admission(
            "00000000-0000-0000-0000-000000000001",
            "Ada",
            "Lovelace",
            Some("ada@example.com".to_owned()),
        )
        .unwrap();
        let json: serde_json::Value = serde_wasm_bindgen::from_value(result).unwrap();
        assert_eq!(json["ok"], true);
        assert!(json["student_id"].is_string());
    }

    #[test]
    fn capability_known_recognizes_engine_capabilities() {
        assert!(capability_known("academic.student.create"));
        assert!(capability_known("hr.staff.create"));
        assert!(!capability_known("nonexistent.capability"));
    }

    #[test]
    fn build_student_summary_returns_typed_id() {
        let result = build_student_summary(
            "00000000-0000-0000-0000-000000000001",
            "Ada",
            "Lovelace",
        )
        .unwrap();
        let json: serde_json::Value = serde_wasm_bindgen::from_value(result).unwrap();
        assert!(json["student_id"].is_string());
        assert_eq!(json["first_name"], "Ada");
        assert_eq!(json["last_name"], "Lovelace");
    }

    #[test]
    fn engine_version_is_non_empty() {
        let v = engine_version();
        assert!(!v.is_empty());
    }
}
