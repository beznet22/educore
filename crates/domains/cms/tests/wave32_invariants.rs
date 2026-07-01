//! Wave 32 invariant enforcement tests for the CMS domain.
//!
//! Pins the four missing invariants surfaced in
//! `docs/audit_reports/stub_vs_implementation.md` ## cms —
//! Deep Invariant Audit:
//!
//! - P4: at most one `Page` per school has `home_page = true`
//! - NC2: `NewsCategory.category_name` unique within school
//! - CT3: `ContentType.type_name` unique within school
//! - FP3: `FrontendPage.slug` unique within school when set
//!
//! Each helper is exposed on the existing service struct
//! (`PageService`, `NewsService`, `ContentService`) and is
//! pure (caller-supplied list of existing rows), so the
//! tests below do not require a repository.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_cms::prelude::*;
use educore_cms::services::{ContentService, NewsService, PageService};

// =============================================================================
// P4: at most one Page per school has home_page = true
// =============================================================================

#[test]
fn wave32_p4_home_page_uniqueness_allows_first_home_page() {
    let result = PageService::check_home_page_uniqueness(0, true);
    assert!(
        result.is_ok(),
        "first home page must be accepted (no existing home pages)"
    );
}

#[test]
fn wave32_p4_home_page_uniqueness_rejects_second_home_page() {
    let err = PageService::check_home_page_uniqueness(1, true)
        .expect_err("second home page must be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("home page") && msg.contains("at most one"),
        "unexpected error message: {msg}"
    );
}

#[test]
fn wave32_p4_home_page_uniqueness_allows_non_home_with_existing_home() {
    let result = PageService::check_home_page_uniqueness(1, false);
    assert!(result.is_ok(), "non-home page is allowed even when home exists");
}

// =============================================================================
// NC2: NewsCategory.category_name unique within school
// =============================================================================

#[test]
fn wave32_nc2_category_name_unique_accepts_new_name() {
    let name = CategoryName::new("announcements").expect("non-empty name");
    let existing: Vec<CategoryName> = vec![];
    let result = NewsService::validate_category_name_unique(&name, &existing);
    assert!(result.is_ok(), "new category name must be accepted");
}

#[test]
fn wave32_nc2_category_name_unique_rejects_duplicate() {
    let name = CategoryName::new("sports").expect("non-empty name");
    let existing = vec![name.clone()];
    let err = NewsService::validate_category_name_unique(&name, &existing)
        .expect_err("duplicate must be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("sports") && msg.contains("already exists"),
        "unexpected error message: {msg}"
    );
}

// =============================================================================
// CT3: ContentType.type_name unique within school
// =============================================================================

#[test]
fn wave32_ct3_content_type_name_unique_accepts_new_name() {
    let name = ContentTypeName::new("syllabus").expect("non-empty name");
    let existing: Vec<ContentTypeName> = vec![];
    let result = ContentService::validate_content_type_name_unique(&name, &existing);
    assert!(result.is_ok(), "new type name must be accepted");
}

#[test]
fn wave32_ct3_content_type_name_unique_rejects_duplicate() {
    let name = ContentTypeName::new("homework").expect("non-empty name");
    let existing = vec![name.clone()];
    let err = ContentService::validate_content_type_name_unique(&name, &existing)
        .expect_err("duplicate must be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("homework") && msg.contains("already exists"),
        "unexpected error message: {msg}"
    );
}

// =============================================================================
// FP3: FrontendPage.slug unique within school when set
// =============================================================================

#[test]
fn wave32_fp3_frontend_page_slug_unique_accepts_new_slug() {
    let slug = Slug::new("about-us").expect("valid slug");
    let existing: Vec<Slug> = vec![];
    assert!(PageService::validate_frontend_page_slug_unique(&slug, &existing));
}

#[test]
fn wave32_fp3_frontend_page_slug_unique_rejects_duplicate() {
    let slug = Slug::new("contact").expect("valid slug");
    let existing = vec![slug.clone()];
    assert!(!PageService::validate_frontend_page_slug_unique(&slug, &existing));
}
