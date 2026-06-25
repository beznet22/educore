//! # Library domain repository ports
//!
//! The repository traits the storage adapters implement. Every
//! repository takes a `SchoolId` (or operates on a typed
//! identifier that already embeds it) and refuses to return
//! data from another school. Tenant isolation is structural.
//!
//! All 9 headline aggregates have a `pub trait
//! XxxRepository: Send + Sync` port trait with the standard
//! `get` / `list` / `insert` / `update` / `delete` methods plus
//! per-aggregate `find_by_*` and `list_for_*` helpers.
//!
//! Mirrors the Phase 7 finance pattern (44 port traits shipped
//! in commit `3fe575e`) and the Phase 8 facilities pattern.

#![allow(missing_docs)]
#![allow(unused_imports)]

use async_trait::async_trait;
use chrono::NaiveDate;

use educore_core::error::Result;
use educore_core::ids::SchoolId;

use crate::aggregate::{
    Book, BookAcquisition, BookCatalogEntry, BookCategory, BookIssue, BookReturn, Fine,
    LibraryMember, LibraryMemberNote,
};
use crate::value_objects::{
    AcademicYearId, BookAcquisitionId, BookCatalogEntryId, BookCategoryId, BookId, BookIssueId,
    BookReturnId, FineId, LibraryMemberId, LibraryMemberNoteId, MemberId, RoleId, StockCopies,
};

// =============================================================================
// BookCategoryRepository
// =============================================================================

/// Port for the `BookCategory` aggregate.
#[async_trait]
pub trait BookCategoryRepository: Send + Sync {
    /// Fetch a category by its typed id.
    async fn get(&self, id: BookCategoryId) -> Result<Option<BookCategory>>;
    /// List all categories for a school.
    async fn list(&self, school: SchoolId) -> Result<Vec<BookCategory>>;
    /// Find a category by its name within a school.
    async fn find_by_name(&self, school: SchoolId, name: &str) -> Result<Option<BookCategory>>;
    /// Insert a new category.
    async fn insert(&self, c: &BookCategory) -> Result<()>;
    /// Update an existing category.
    async fn update(&self, c: &BookCategory) -> Result<()>;
    /// Delete (soft) a category.
    async fn delete(&self, id: BookCategoryId) -> Result<()>;
}

// =============================================================================
// BookRepository
// =============================================================================

/// Port for the `Book` aggregate.
#[async_trait]
pub trait BookRepository: Send + Sync {
    /// Fetch a book by its typed id.
    async fn get(&self, id: BookId) -> Result<Option<Book>>;
    /// Fetch a book by ISBN within a school.
    async fn get_by_isbn(&self, school: SchoolId, isbn: &str) -> Result<Option<Book>>;
    /// Fetch a book by cataloguing number within a school.
    async fn get_by_book_number(&self, school: SchoolId, book_number: &str)
        -> Result<Option<Book>>;
    /// List all books for a school in a given academic year.
    async fn list(&self, school: SchoolId, year: AcademicYearId) -> Result<Vec<Book>>;
    /// List all books for a school in a given category.
    async fn list_for_category(
        &self,
        school: SchoolId,
        category: BookCategoryId,
    ) -> Result<Vec<Book>>;
    /// Search the catalog (free-text query, optional category).
    async fn search(
        &self,
        school: SchoolId,
        query: &str,
        category: Option<BookCategoryId>,
        limit: u32,
    ) -> Result<Vec<Book>>;
    /// Insert a new book.
    async fn insert(&self, book: &Book) -> Result<()>;
    /// Update an existing book.
    async fn update(&self, book: &Book) -> Result<()>;
    /// Delete (soft) a book.
    async fn delete(&self, id: BookId) -> Result<()>;
    /// Adjust a book's stock quantity atomically. The storage
    /// adapter enforces the invariant
    /// `new_quantity >= sum(open_issue_quantities)`.
    async fn adjust_quantity(&self, id: BookId, new_quantity: StockCopies) -> Result<()>;
}

// =============================================================================
// LibraryMemberRepository
// =============================================================================

/// Port for the `LibraryMember` aggregate.
#[async_trait]
pub trait LibraryMemberRepository: Send + Sync {
    /// Fetch a member by its typed id.
    async fn get(&self, id: LibraryMemberId) -> Result<Option<LibraryMember>>;
    /// Find a member by (member_type, member) within a
    /// (school, year).
    async fn find(
        &self,
        school: SchoolId,
        year: AcademicYearId,
        member: MemberId,
        member_type: RoleId,
    ) -> Result<Option<LibraryMember>>;
    /// List all members for a school in a given year.
    async fn list(&self, school: SchoolId, year: AcademicYearId) -> Result<Vec<LibraryMember>>;
    /// List all active members for a school in a given year.
    async fn list_active(
        &self,
        school: SchoolId,
        year: AcademicYearId,
    ) -> Result<Vec<LibraryMember>>;
    /// Insert a new member.
    async fn insert(&self, m: &LibraryMember) -> Result<()>;
    /// Update an existing member.
    async fn update(&self, m: &LibraryMember) -> Result<()>;
    /// Deactivate a member.
    async fn deactivate(&self, id: LibraryMemberId) -> Result<()>;
    /// Reactivate a member.
    async fn reactivate(&self, id: LibraryMemberId) -> Result<()>;
    /// Delete (soft) a member.
    async fn delete(&self, id: LibraryMemberId) -> Result<()>;
}

// =============================================================================
// BookIssueRepository
// =============================================================================

/// Port for the `BookIssue` aggregate.
#[async_trait]
pub trait BookIssueRepository: Send + Sync {
    /// Fetch an issue by its typed id.
    async fn get(&self, id: BookIssueId) -> Result<Option<BookIssue>>;
    /// List all issues for a member.
    async fn list_for_member(&self, member: LibraryMemberId) -> Result<Vec<BookIssue>>;
    /// List all issues for a book.
    async fn list_for_book(&self, book: BookId) -> Result<Vec<BookIssue>>;
    /// List all open issues for a school as of `as_of`.
    async fn list_open(&self, school: SchoolId, as_of: NaiveDate) -> Result<Vec<BookIssue>>;
    /// List all overdue issues for a school as of `as_of`.
    async fn list_overdue(&self, school: SchoolId, as_of: NaiveDate) -> Result<Vec<BookIssue>>;
    /// The sum of quantities on open issues for a book.
    /// Used by `adjust_quantity` to enforce the "open issues <=
    /// quantity" invariant atomically.
    async fn open_quantity_for_book(&self, book: BookId) -> Result<u32>;
    /// Insert a new issue.
    async fn insert(&self, issue: &BookIssue) -> Result<()>;
    /// Update an existing issue.
    async fn update(&self, issue: &BookIssue) -> Result<()>;
}

// =============================================================================
// BookReturnRepository
// =============================================================================

/// Port for the `BookReturn` aggregate.
#[async_trait]
pub trait BookReturnRepository: Send + Sync {
    /// Fetch a return by its typed id.
    async fn get(&self, id: BookReturnId) -> Result<Option<BookReturn>>;
    /// List all returns for a book.
    async fn list_for_book(&self, book: BookId) -> Result<Vec<BookReturn>>;
    /// List all returns for a member.
    async fn list_for_member(&self, member: LibraryMemberId) -> Result<Vec<BookReturn>>;
    /// List returns for a school in the inclusive date range
    /// `[from, to]`.
    async fn list_for_date_range(
        &self,
        school: SchoolId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<BookReturn>>;
    /// Insert a new return.
    async fn insert(&self, r: &BookReturn) -> Result<()>;
}

// =============================================================================
// FineRepository
// =============================================================================

/// Port for the `Fine` aggregate.
#[async_trait]
pub trait FineRepository: Send + Sync {
    /// Fetch a fine by its typed id.
    async fn get(&self, id: FineId) -> Result<Option<Fine>>;
    /// List all fines for a book issue.
    async fn list_for_issue(&self, issue: BookIssueId) -> Result<Vec<Fine>>;
    /// List all open (non-waived) fines for a member.
    async fn list_open_for_member(&self, member: LibraryMemberId) -> Result<Vec<Fine>>;
    /// List all fines for a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<Fine>>;
    /// Insert a new fine.
    async fn insert(&self, f: &Fine) -> Result<()>;
    /// Update an existing fine (used by the waiver path).
    async fn update(&self, f: &Fine) -> Result<()>;
}

// 7.
// =============================================================================
// BookAcquisitionRepository
// =============================================================================

/// Port for the `BookAcquisition` aggregate.
#[async_trait]
pub trait BookAcquisitionRepository: Send + Sync {
    /// Fetch an acquisition by its typed id.
    async fn get(&self, id: BookAcquisitionId) -> Result<Option<BookAcquisition>>;
    /// List all acquisitions for a book (oldest first).
    async fn list_for_book(&self, book: BookId) -> Result<Vec<BookAcquisition>>;
    /// List all acquisitions for a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<BookAcquisition>>;
    /// Insert a new acquisition.
    async fn insert(&self, a: &BookAcquisition) -> Result<()>;
    /// Update an existing acquisition.
    async fn update(&self, a: &BookAcquisition) -> Result<()>;
    /// Delete (soft) an acquisition.
    async fn delete(&self, id: BookAcquisitionId) -> Result<()>;
}

// 8.
// =============================================================================
// BookCatalogEntryRepository
// =============================================================================

/// Port for the `BookCatalogEntry` aggregate (append-only
/// catalog history).
#[async_trait]
pub trait BookCatalogEntryRepository: Send + Sync {
    /// Fetch a catalog entry by its typed id.
    async fn get(&self, id: BookCatalogEntryId) -> Result<Option<BookCatalogEntry>>;
    /// List all catalog entries for a book (oldest first; the
    /// full versioned history).
    async fn list_for_book(&self, book: BookId) -> Result<Vec<BookCatalogEntry>>;
    /// Fetch the most recent catalog entry for a book (the
    /// current snapshot).
    async fn latest_for_book(&self, book: BookId) -> Result<Option<BookCatalogEntry>>;
    /// Insert a new catalog entry.
    async fn insert(&self, e: &BookCatalogEntry) -> Result<()>;
    /// Update an existing catalog entry.
    async fn update(&self, e: &BookCatalogEntry) -> Result<()>;
    /// Delete (soft) a catalog entry.
    async fn delete(&self, id: BookCatalogEntryId) -> Result<()>;
}

// 9.
// =============================================================================
// LibraryMemberNoteRepository
// =============================================================================

/// Port for the `LibraryMemberNote` aggregate.
#[async_trait]
pub trait LibraryMemberNoteRepository: Send + Sync {
    /// Fetch a note by its typed id.
    async fn get(&self, id: LibraryMemberNoteId) -> Result<Option<LibraryMemberNote>>;
    /// List all notes for a member (oldest first; the full
    /// administrative history).
    async fn list_for_member(&self, member: LibraryMemberId) -> Result<Vec<LibraryMemberNote>>;
    /// List the notes for a member that are visible to the
    /// member (i.e. `visible_to_member == true`).
    async fn list_visible_for_member(
        &self,
        member: LibraryMemberId,
    ) -> Result<Vec<LibraryMemberNote>>;
    /// Insert a new note.
    async fn insert(&self, n: &LibraryMemberNote) -> Result<()>;
    /// Update an existing note.
    async fn update(&self, n: &LibraryMemberNote) -> Result<()>;
    /// Delete (soft) a note.
    async fn delete(&self, id: LibraryMemberNoteId) -> Result<()>;
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    //! The repository traits are object-safe. The smoke tests
    //! assert `Box<dyn XxxRepository>` compiles.

    use super::*;

    fn _object_safety_check_book_category() -> Box<dyn BookCategoryRepository> {
        struct Impl;
        #[async_trait]
        impl BookCategoryRepository for Impl {
            async fn get(&self, _id: BookCategoryId) -> Result<Option<BookCategory>> {
                unreachable!()
            }
            async fn list(&self, _school: SchoolId) -> Result<Vec<BookCategory>> {
                unreachable!()
            }
            async fn find_by_name(
                &self,
                _school: SchoolId,
                _name: &str,
            ) -> Result<Option<BookCategory>> {
                unreachable!()
            }
            async fn insert(&self, _c: &BookCategory) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _c: &BookCategory) -> Result<()> {
                unreachable!()
            }
            async fn delete(&self, _id: BookCategoryId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_book() -> Box<dyn BookRepository> {
        struct Impl;
        #[async_trait]
        impl BookRepository for Impl {
            async fn get(&self, _id: BookId) -> Result<Option<Book>> {
                unreachable!()
            }
            async fn get_by_isbn(&self, _school: SchoolId, _isbn: &str) -> Result<Option<Book>> {
                unreachable!()
            }
            async fn get_by_book_number(
                &self,
                _school: SchoolId,
                _book_number: &str,
            ) -> Result<Option<Book>> {
                unreachable!()
            }
            async fn list(&self, _school: SchoolId, _year: AcademicYearId) -> Result<Vec<Book>> {
                unreachable!()
            }
            async fn list_for_category(
                &self,
                _school: SchoolId,
                _category: BookCategoryId,
            ) -> Result<Vec<Book>> {
                unreachable!()
            }
            async fn search(
                &self,
                _school: SchoolId,
                _query: &str,
                _category: Option<BookCategoryId>,
                _limit: u32,
            ) -> Result<Vec<Book>> {
                unreachable!()
            }
            async fn insert(&self, _book: &Book) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _book: &Book) -> Result<()> {
                unreachable!()
            }
            async fn delete(&self, _id: BookId) -> Result<()> {
                unreachable!()
            }
            async fn adjust_quantity(&self, _id: BookId, _new_quantity: StockCopies) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_library_member() -> Box<dyn LibraryMemberRepository> {
        struct Impl;
        #[async_trait]
        impl LibraryMemberRepository for Impl {
            async fn get(&self, _id: LibraryMemberId) -> Result<Option<LibraryMember>> {
                unreachable!()
            }
            async fn find(
                &self,
                _school: SchoolId,
                _year: AcademicYearId,
                _member: MemberId,
                _member_type: RoleId,
            ) -> Result<Option<LibraryMember>> {
                unreachable!()
            }
            async fn list(
                &self,
                _school: SchoolId,
                _year: AcademicYearId,
            ) -> Result<Vec<LibraryMember>> {
                unreachable!()
            }
            async fn list_active(
                &self,
                _school: SchoolId,
                _year: AcademicYearId,
            ) -> Result<Vec<LibraryMember>> {
                unreachable!()
            }
            async fn insert(&self, _m: &LibraryMember) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _m: &LibraryMember) -> Result<()> {
                unreachable!()
            }
            async fn deactivate(&self, _id: LibraryMemberId) -> Result<()> {
                unreachable!()
            }
            async fn reactivate(&self, _id: LibraryMemberId) -> Result<()> {
                unreachable!()
            }
            async fn delete(&self, _id: LibraryMemberId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_book_issue() -> Box<dyn BookIssueRepository> {
        struct Impl;
        #[async_trait]
        impl BookIssueRepository for Impl {
            async fn get(&self, _id: BookIssueId) -> Result<Option<BookIssue>> {
                unreachable!()
            }
            async fn list_for_member(&self, _member: LibraryMemberId) -> Result<Vec<BookIssue>> {
                unreachable!()
            }
            async fn list_for_book(&self, _book: BookId) -> Result<Vec<BookIssue>> {
                unreachable!()
            }
            async fn list_open(
                &self,
                _school: SchoolId,
                _as_of: NaiveDate,
            ) -> Result<Vec<BookIssue>> {
                unreachable!()
            }
            async fn list_overdue(
                &self,
                _school: SchoolId,
                _as_of: NaiveDate,
            ) -> Result<Vec<BookIssue>> {
                unreachable!()
            }
            async fn open_quantity_for_book(&self, _book: BookId) -> Result<u32> {
                unreachable!()
            }
            async fn insert(&self, _issue: &BookIssue) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _issue: &BookIssue) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_book_return() -> Box<dyn BookReturnRepository> {
        struct Impl;
        #[async_trait]
        impl BookReturnRepository for Impl {
            async fn get(&self, _id: BookReturnId) -> Result<Option<BookReturn>> {
                unreachable!()
            }
            async fn list_for_book(&self, _book: BookId) -> Result<Vec<BookReturn>> {
                unreachable!()
            }
            async fn list_for_member(&self, _member: LibraryMemberId) -> Result<Vec<BookReturn>> {
                unreachable!()
            }
            async fn list_for_date_range(
                &self,
                _school: SchoolId,
                _from: NaiveDate,
                _to: NaiveDate,
            ) -> Result<Vec<BookReturn>> {
                unreachable!()
            }
            async fn insert(&self, _r: &BookReturn) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_fine() -> Box<dyn FineRepository> {
        struct Impl;
        #[async_trait]
        impl FineRepository for Impl {
            async fn get(&self, _id: FineId) -> Result<Option<Fine>> {
                unreachable!()
            }
            async fn list_for_issue(&self, _issue: BookIssueId) -> Result<Vec<Fine>> {
                unreachable!()
            }
            async fn list_open_for_member(&self, _member: LibraryMemberId) -> Result<Vec<Fine>> {
                unreachable!()
            }
            async fn list_for_school(&self, _school: SchoolId) -> Result<Vec<Fine>> {
                unreachable!()
            }
            async fn insert(&self, _f: &Fine) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _f: &Fine) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_book_acquisition() -> Box<dyn BookAcquisitionRepository> {
        struct Impl;
        #[async_trait]
        impl BookAcquisitionRepository for Impl {
            async fn get(&self, _id: BookAcquisitionId) -> Result<Option<BookAcquisition>> {
                unreachable!()
            }
            async fn list_for_book(&self, _book: BookId) -> Result<Vec<BookAcquisition>> {
                unreachable!()
            }
            async fn list_for_school(&self, _school: SchoolId) -> Result<Vec<BookAcquisition>> {
                unreachable!()
            }
            async fn insert(&self, _a: &BookAcquisition) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _a: &BookAcquisition) -> Result<()> {
                unreachable!()
            }
            async fn delete(&self, _id: BookAcquisitionId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_book_catalog_entry() -> Box<dyn BookCatalogEntryRepository> {
        struct Impl;
        #[async_trait]
        impl BookCatalogEntryRepository for Impl {
            async fn get(&self, _id: BookCatalogEntryId) -> Result<Option<BookCatalogEntry>> {
                unreachable!()
            }
            async fn list_for_book(&self, _book: BookId) -> Result<Vec<BookCatalogEntry>> {
                unreachable!()
            }
            async fn latest_for_book(&self, _book: BookId) -> Result<Option<BookCatalogEntry>> {
                unreachable!()
            }
            async fn insert(&self, _e: &BookCatalogEntry) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _e: &BookCatalogEntry) -> Result<()> {
                unreachable!()
            }
            async fn delete(&self, _id: BookCatalogEntryId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_library_member_note() -> Box<dyn LibraryMemberNoteRepository> {
        struct Impl;
        #[async_trait]
        impl LibraryMemberNoteRepository for Impl {
            async fn get(&self, _id: LibraryMemberNoteId) -> Result<Option<LibraryMemberNote>> {
                unreachable!()
            }
            async fn list_for_member(
                &self,
                _member: LibraryMemberId,
            ) -> Result<Vec<LibraryMemberNote>> {
                unreachable!()
            }
            async fn list_visible_for_member(
                &self,
                _member: LibraryMemberId,
            ) -> Result<Vec<LibraryMemberNote>> {
                unreachable!()
            }
            async fn insert(&self, _n: &LibraryMemberNote) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _n: &LibraryMemberNote) -> Result<()> {
                unreachable!()
            }
            async fn delete(&self, _id: LibraryMemberNoteId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    #[test]
    fn repository_traits_are_object_safe() {
        // If the traits were not object-safe, these fn definitions
        // would fail to compile.
        let _cat: Box<dyn BookCategoryRepository> = _object_safety_check_book_category();
        let _book: Box<dyn BookRepository> = _object_safety_check_book();
        let _member: Box<dyn LibraryMemberRepository> = _object_safety_check_library_member();
        let _issue: Box<dyn BookIssueRepository> = _object_safety_check_book_issue();
        let _ret: Box<dyn BookReturnRepository> = _object_safety_check_book_return();
        let _fine: Box<dyn FineRepository> = _object_safety_check_fine();
        let _acq: Box<dyn BookAcquisitionRepository> = _object_safety_check_book_acquisition();
        let _cat_entry: Box<dyn BookCatalogEntryRepository> =
            _object_safety_check_book_catalog_entry();
        let _note: Box<dyn LibraryMemberNoteRepository> =
            _object_safety_check_library_member_note();
    }
}
