//! Integration tests for the `#[derive(DomainQuery)]` proc macro.
//!
//! Per Cargo's convention, tests for proc-macro crates live in a
//! `tests/` directory so they're compiled as a separate crate. This
//! lets us exercise the macro without the inline-test cfg guards
//! that conflict with the proc-macro crate's library target.

// Test scaffolding: relax the workspace lint baseline (which forbids
// `unwrap`/`expect`/`panic` in production) so the assertions read
// naturally. The macro-generated code is exercised via `.unwrap()`
// on the `Result` returned by `build_query_node`, not via any
// internal `unwrap` of the macro itself.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::prelude::*;
use educore_query_derive::DomainQuery;
use uuid::Uuid;

// ---- Test struct #1: filterable + sortable, no relations ----
#[derive(DomainQuery)]
pub struct Note {
    pub id: Uuid,
    #[query(sortable)]
    pub title: String,
    #[query(filterable)]
    pub body: String,
    #[query(sortable)]
    pub created_at: String,
}

// ---- Test struct #2: relations present ----
#[derive(DomainQuery)]
pub struct Parent {
    pub id: Uuid,
    #[query(filterable)]
    pub city: String,
}

#[derive(DomainQuery)]
pub struct Student {
    pub id: Uuid,
    #[query(sortable)]
    pub last_name: String,
    #[query(filterable)]
    pub status: String,
    #[query(filterable, relation = "Parent", builder = "ParentQueryBuilder")]
    pub parent_id: Uuid,
}

// ---- Test struct #3: no queryable fields (relation only) ----
#[derive(DomainQuery)]
pub struct Bookmark {
    pub id: Uuid,
    #[query(relation = "Note", builder = "NoteQueryBuilder")]
    pub note_id: Uuid,
}

#[test]
fn field_enum_column_names_match_field_name() {
    assert_eq!(NoteField::Title.column_name(), "title");
    assert_eq!(NoteField::Body.column_name(), "body");
    assert_eq!(NoteField::CreatedAt.column_name(), "created_at");
}

#[test]
fn field_enum_variants_in_declaration_order() {
    assert_eq!(
        NoteField::all_variants(),
        &[NoteField::Title, NoteField::Body, NoteField::CreatedAt]
    );
}

#[test]
fn builder_new_is_empty() {
    let b = NoteQueryBuilder::new();
    assert_eq!(b.school_id(), None);
}

#[test]
fn builder_for_school_sets_school_id() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let b = NoteQueryBuilder::new().for_school(school);
    assert_eq!(b.school_id(), Some(school));
}

#[test]
fn builder_without_school_id_fails_build_query_node() {
    let b = NoteQueryBuilder::new();
    let result = b.build_query_node();
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn builder_with_school_id_succeeds_build_query_node() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let (_node, _page) = NoteQueryBuilder::new()
        .for_school(school)
        .build_query_node()
        .unwrap();
}

#[test]
fn builder_where_eq_compiles_and_builds() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let (_node, _page) = NoteQueryBuilder::new()
        .for_school(school)
        .where_eq(NoteField::Title, "hello")
        .build_query_node()
        .unwrap();
}

#[test]
fn builder_order_by_appends_order() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let b = NoteQueryBuilder::new()
        .for_school(school)
        .order_by(NoteField::Title);
    let orders = b.orders();
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].field, NoteField::Title);
}

#[test]
fn builder_limit_offset_page() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let (_node, page) = NoteQueryBuilder::new()
        .for_school(school)
        .page(20, 50)
        .build_query_node()
        .unwrap();
    assert_eq!(page.offset, 20);
    assert_eq!(page.limit, 50);
}

#[test]
fn builder_default_limit_is_50() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let (_node, page) = NoteQueryBuilder::new()
        .for_school(school)
        .build_query_node()
        .unwrap();
    assert_eq!(page.limit, 50);
}

#[test]
fn struct_query_returns_builder() {
    let _b: NoteQueryBuilder = Note::query();
}

#[test]
fn relation_enum_has_one_variant_per_relation() {
    // `all_relations` lives on the field enum (where the
    // HasRelations trait is implemented), not on the relation
    // enum itself.
    assert_eq!(StudentField::all_relations(), &[StudentRelation::Parent]);
}

#[test]
fn relation_into_relation_gives_typed_relation() {
    let rel: educore_core::query::Relation = StudentRelation::Parent.into();
    assert_eq!(rel.name, "parent");
    assert_eq!(rel.id, 0);
}

#[test]
fn has_relations_trait_works() {
    fn assert_has_relations<T: educore_core::query::HasRelations>() {}
    assert_has_relations::<StudentField>();
    assert_has_relations::<BookmarkField>();
}

#[test]
fn where_has_typed_method_compiles() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let _b = StudentQueryBuilder::new()
        .for_school(school)
        .where_has_Parent(|p| p.where_eq(ParentField::City, "Boston"));
}

#[test]
fn builder_with_appends_relation() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let b = StudentQueryBuilder::new()
        .for_school(school)
        .with(StudentRelation::Parent);
    let rels: Vec<StudentRelation> = b.relations().collect();
    assert_eq!(rels, vec![StudentRelation::Parent]);
}

#[test]
fn no_relations_struct_compiles() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let b = BookmarkQueryBuilder::new()
        .for_school(school)
        .where_has_Note(|n| n.where_eq(NoteField::Title, "x"));
    let rels: Vec<BookmarkRelation> = b.relations().collect();
    assert!(rels.is_empty());
}

#[test]
fn column_name_includes_underscores() {
    assert_eq!(StudentField::LastName.column_name(), "last_name");
    // `parent_id` is a relation field, not a queryable field, so it's
    // accessed as `StudentRelation::Parent` (not `StudentField::ParentId`).
    // The column name is still "parent_id" via the underlying
    // `Parent` aggregate's schema.
}

#[test]
fn builder_with_many_appends_multiple_relations() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let b = BookmarkQueryBuilder::new()
        .for_school(school)
        .with_many(&[BookmarkRelation::Note]);
    let rels: Vec<BookmarkRelation> = b.relations().collect();
    assert_eq!(rels, vec![BookmarkRelation::Note]);
}
