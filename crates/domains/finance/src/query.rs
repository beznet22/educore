//! # Finance query stubs
//!
//! Phase 7 query stubs for the finance aggregates. Every
//! `execute()` returns `Err(DomainError::not_supported(...))`
//! until Phase 17 wires the typed executor + storage-port
//! translation.

#![allow(missing_docs)]
#![allow(dead_code)]

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;

use crate::aggregate::{
    BankAccount, BankStatement, Expense, FeesCarryForward, FeesInvoice, FeesPayment, Income,
    PayrollPayment, Transaction, Wallet, WalletTransaction,
};
use crate::value_objects::{
    AccountType, ApprovalStatus, BankAccountId, ExpenseHeadId, FeesInvoiceStatus, IncomeHeadId,
    StatementType, WalletId, WalletTransactionId, WalletTxType,
};

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WalletQuery {
    pub user_id: Option<educore_core::ids::UserId>,
    pub min_balance_minor: Option<i64>,
    pub max_balance_minor: Option<i64>,
    pub offset: u32,
    pub limit: u32,
}

impl WalletQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            user_id: None,
            min_balance_minor: None,
            max_balance_minor: None,
            offset: 0,
            limit: 50,
        }
    }

    /// Phase 7 stub.
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<Wallet>> {
        Err(DomainError::not_supported(
            "WalletQuery::execute is a Phase 7 stub",
        ))
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WalletTransactionQuery {
    pub wallet_id: Option<WalletId>,
    pub user_id: Option<educore_core::ids::UserId>,
    pub wallet_type: Option<WalletTxType>,
    pub status: Option<ApprovalStatus>,
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
    pub offset: u32,
    pub limit: u32,
}

impl WalletTransactionQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            wallet_id: None,
            user_id: None,
            wallet_type: None,
            status: None,
            from: None,
            to: None,
            offset: 0,
            limit: 50,
        }
    }

    /// Phase 7 stub.
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<WalletTransaction>> {
        Err(DomainError::not_supported(
            "WalletTransactionQuery::execute is a Phase 7 stub",
        ))
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FeesPaymentQuery {
    pub student_id: Option<educore_academic::StudentId>,
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
    pub offset: u32,
    pub limit: u32,
}

impl FeesPaymentQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            student_id: None,
            from: None,
            to: None,
            offset: 0,
            limit: 50,
        }
    }

    /// Phase 7 stub.
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<FeesPayment>> {
        Err(DomainError::not_supported(
            "FeesPaymentQuery::execute is a Phase 7 stub",
        ))
    }
}

// =============================================================================
// Phase 7 query stubs (continued): the remaining finance aggregates.
// Each `execute()` returns a Phase 7 stub error until the typed
// executor lands in Phase 17.
// =============================================================================

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FeesInvoiceQuery {
    pub school_id: Option<SchoolId>,
    pub status: Option<FeesInvoiceStatus>,
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
    pub offset: u32,
    pub limit: u32,
}

impl FeesInvoiceQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            school_id: None,
            status: None,
            from: None,
            to: None,
            offset: 0,
            limit: 50,
        }
    }

    #[must_use]
    pub const fn with_school(mut self, school: SchoolId) -> Self {
        self.school_id = Some(school);
        self
    }

    #[must_use]
    pub const fn with_status(mut self, status: FeesInvoiceStatus) -> Self {
        self.status = Some(status);
        self
    }

    #[must_use]
    pub const fn with_from(mut self, from: chrono::NaiveDate) -> Self {
        self.from = Some(from);
        self
    }

    #[must_use]
    pub const fn with_to(mut self, to: chrono::NaiveDate) -> Self {
        self.to = Some(to);
        self
    }

    #[must_use]
    pub const fn with_offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    #[must_use]
    pub const fn with_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    /// Phase 7 stub.
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<FeesInvoice>> {
        Err(DomainError::not_supported(
            "FeesInvoiceQuery::execute is a Phase 7 stub",
        ))
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExpenseQuery {
    pub school_id: Option<SchoolId>,
    pub expense_head_id: Option<ExpenseHeadId>,
    pub account_id: Option<BankAccountId>,
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
    pub offset: u32,
    pub limit: u32,
}

impl ExpenseQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            school_id: None,
            expense_head_id: None,
            account_id: None,
            from: None,
            to: None,
            offset: 0,
            limit: 50,
        }
    }

    #[must_use]
    pub const fn with_school(mut self, school: SchoolId) -> Self {
        self.school_id = Some(school);
        self
    }

    #[must_use]
    pub const fn with_expense_head(mut self, head: ExpenseHeadId) -> Self {
        self.expense_head_id = Some(head);
        self
    }

    #[must_use]
    pub const fn with_account(mut self, account: BankAccountId) -> Self {
        self.account_id = Some(account);
        self
    }

    #[must_use]
    pub const fn with_from(mut self, from: chrono::NaiveDate) -> Self {
        self.from = Some(from);
        self
    }

    #[must_use]
    pub const fn with_to(mut self, to: chrono::NaiveDate) -> Self {
        self.to = Some(to);
        self
    }

    #[must_use]
    pub const fn with_offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    #[must_use]
    pub const fn with_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    /// Phase 7 stub.
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<Expense>> {
        Err(DomainError::not_supported(
            "ExpenseQuery::execute is a Phase 7 stub",
        ))
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IncomeQuery {
    pub school_id: Option<SchoolId>,
    pub income_head_id: Option<IncomeHeadId>,
    pub account_id: Option<BankAccountId>,
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
    pub offset: u32,
    pub limit: u32,
}

impl IncomeQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            school_id: None,
            income_head_id: None,
            account_id: None,
            from: None,
            to: None,
            offset: 0,
            limit: 50,
        }
    }

    #[must_use]
    pub const fn with_school(mut self, school: SchoolId) -> Self {
        self.school_id = Some(school);
        self
    }

    #[must_use]
    pub const fn with_income_head(mut self, head: IncomeHeadId) -> Self {
        self.income_head_id = Some(head);
        self
    }

    #[must_use]
    pub const fn with_account(mut self, account: BankAccountId) -> Self {
        self.account_id = Some(account);
        self
    }

    #[must_use]
    pub const fn with_from(mut self, from: chrono::NaiveDate) -> Self {
        self.from = Some(from);
        self
    }

    #[must_use]
    pub const fn with_to(mut self, to: chrono::NaiveDate) -> Self {
        self.to = Some(to);
        self
    }

    #[must_use]
    pub const fn with_offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    #[must_use]
    pub const fn with_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    /// Phase 7 stub.
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<Income>> {
        Err(DomainError::not_supported(
            "IncomeQuery::execute is a Phase 7 stub",
        ))
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BankStatementQuery {
    pub school_id: Option<SchoolId>,
    pub bank_id: Option<BankAccountId>,
    pub statement_type: Option<StatementType>,
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
    pub offset: u32,
    pub limit: u32,
}

impl BankStatementQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            school_id: None,
            bank_id: None,
            statement_type: None,
            from: None,
            to: None,
            offset: 0,
            limit: 50,
        }
    }

    #[must_use]
    pub const fn with_school(mut self, school: SchoolId) -> Self {
        self.school_id = Some(school);
        self
    }

    #[must_use]
    pub const fn with_bank(mut self, bank: BankAccountId) -> Self {
        self.bank_id = Some(bank);
        self
    }

    #[must_use]
    pub const fn with_statement_type(mut self, kind: StatementType) -> Self {
        self.statement_type = Some(kind);
        self
    }

    #[must_use]
    pub const fn with_from(mut self, from: chrono::NaiveDate) -> Self {
        self.from = Some(from);
        self
    }

    #[must_use]
    pub const fn with_to(mut self, to: chrono::NaiveDate) -> Self {
        self.to = Some(to);
        self
    }

    #[must_use]
    pub const fn with_offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    #[must_use]
    pub const fn with_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    /// Phase 7 stub.
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<BankStatement>> {
        Err(DomainError::not_supported(
            "BankStatementQuery::execute is a Phase 7 stub",
        ))
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PayrollPaymentQuery {
    pub school_id: Option<SchoolId>,
    pub staff_id: Option<crate::value_objects::StaffId>,
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
    pub offset: u32,
    pub limit: u32,
}

impl PayrollPaymentQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            school_id: None,
            staff_id: None,
            from: None,
            to: None,
            offset: 0,
            limit: 50,
        }
    }

    #[must_use]
    pub const fn with_school(mut self, school: SchoolId) -> Self {
        self.school_id = Some(school);
        self
    }

    #[must_use]
    pub const fn with_staff(mut self, staff: crate::value_objects::StaffId) -> Self {
        self.staff_id = Some(staff);
        self
    }

    #[must_use]
    pub const fn with_from(mut self, from: chrono::NaiveDate) -> Self {
        self.from = Some(from);
        self
    }

    #[must_use]
    pub const fn with_to(mut self, to: chrono::NaiveDate) -> Self {
        self.to = Some(to);
        self
    }

    #[must_use]
    pub const fn with_offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    #[must_use]
    pub const fn with_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    /// Phase 7 stub.
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<PayrollPayment>> {
        Err(DomainError::not_supported(
            "PayrollPaymentQuery::execute is a Phase 7 stub",
        ))
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FeesCarryForwardQuery {
    pub school_id: Option<SchoolId>,
    pub student_id: Option<crate::value_objects::StudentId>,
    pub from_academic_id: Option<crate::value_objects::AcademicYearId>,
    pub to_academic_id: Option<crate::value_objects::AcademicYearId>,
    pub offset: u32,
    pub limit: u32,
}

impl FeesCarryForwardQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            school_id: None,
            student_id: None,
            from_academic_id: None,
            to_academic_id: None,
            offset: 0,
            limit: 50,
        }
    }

    #[must_use]
    pub const fn with_school(mut self, school: SchoolId) -> Self {
        self.school_id = Some(school);
        self
    }

    #[must_use]
    pub const fn with_student(mut self, student: crate::value_objects::StudentId) -> Self {
        self.student_id = Some(student);
        self
    }

    #[must_use]
    pub const fn with_from_academic(mut self, year: crate::value_objects::AcademicYearId) -> Self {
        self.from_academic_id = Some(year);
        self
    }

    #[must_use]
    pub const fn with_to_academic(mut self, year: crate::value_objects::AcademicYearId) -> Self {
        self.to_academic_id = Some(year);
        self
    }

    #[must_use]
    pub const fn with_offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    #[must_use]
    pub const fn with_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    /// Phase 7 stub.
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<FeesCarryForward>> {
        Err(DomainError::not_supported(
            "FeesCarryForwardQuery::execute is a Phase 7 stub",
        ))
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BankAccountQuery {
    pub school_id: Option<SchoolId>,
    pub account_type: Option<AccountType>,
    pub is_active: Option<bool>,
    pub offset: u32,
    pub limit: u32,
}

impl BankAccountQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            school_id: None,
            account_type: None,
            is_active: None,
            offset: 0,
            limit: 50,
        }
    }

    #[must_use]
    pub const fn with_school(mut self, school: SchoolId) -> Self {
        self.school_id = Some(school);
        self
    }

    #[must_use]
    pub const fn with_account_type(mut self, kind: AccountType) -> Self {
        self.account_type = Some(kind);
        self
    }

    #[must_use]
    pub const fn with_is_active(mut self, active: bool) -> Self {
        self.is_active = Some(active);
        self
    }

    #[must_use]
    pub const fn with_offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    #[must_use]
    pub const fn with_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    /// Phase 7 stub.
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<BankAccount>> {
        Err(DomainError::not_supported(
            "BankAccountQuery::execute is a Phase 7 stub",
        ))
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransactionQuery {
    pub school_id: Option<SchoolId>,
    pub morphable_type: Option<String>,
    pub morphable_id: Option<uuid::Uuid>,
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
    pub offset: u32,
    pub limit: u32,
}

impl TransactionQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            school_id: None,
            morphable_type: None,
            morphable_id: None,
            from: None,
            to: None,
            offset: 0,
            limit: 50,
        }
    }

    #[must_use]
    pub const fn with_school(mut self, school: SchoolId) -> Self {
        self.school_id = Some(school);
        self
    }

    #[must_use]
    pub fn with_morphable_type(mut self, kind: String) -> Self {
        self.morphable_type = Some(kind);
        self
    }

    #[must_use]
    pub const fn with_morphable_id(mut self, id: uuid::Uuid) -> Self {
        self.morphable_id = Some(id);
        self
    }

    #[must_use]
    pub const fn with_from(mut self, from: chrono::NaiveDate) -> Self {
        self.from = Some(from);
        self
    }

    #[must_use]
    pub const fn with_to(mut self, to: chrono::NaiveDate) -> Self {
        self.to = Some(to);
        self
    }

    #[must_use]
    pub const fn with_offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    #[must_use]
    pub const fn with_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    /// Phase 7 stub.
    pub async fn execute(&self, _ctx: &TenantContext) -> Result<Vec<Transaction>> {
        Err(DomainError::not_supported(
            "TransactionQuery::execute is a Phase 7 stub",
        ))
    }
}
