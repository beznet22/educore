//! CLI command handlers. Each builds a `TenantContext` and
//! delegates to the in-memory testkit backend.

use std::str::FromStr;

use anyhow::{anyhow, Result};
use educore_core::clock::{IdGenerator, SystemIdGen};
use educore_core::ids::{Identifier, SchoolId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};
use educore_payment::port::{
    CardToken, ChargeRequest, ChequeDate, CurrencyCode, CustomerId, CustomerRef, Money,
    PaymentMethod, PaymentProvider,
};
use educore_storage::{StorageAdapter, StudentAttendanceRow};
use educore_testkit::test_world;
use uuid::Uuid;

use crate::Command;

/// Admit a student. Prints the synthetic student id as JSON.
#[allow(clippy::too_many_arguments)]
pub async fn admit(
    school: String,
    first: String,
    last: String,
    class: String,
    section: String,
) -> Result<()> {
    let _world = test_world();
    let school_id = parse_school(&school)?;
    let class_id = parse_uuid(&class, "class")?;
    let section_id = parse_uuid(&section, "section")?;
    let g = SystemIdGen;
    let user = g.next_user_id();
    let corr = g.next_correlation_id();
    let _ctx = TenantContext::for_user(school_id, user, corr, UserType::SchoolAdmin);

    let student_id = g.next_uuid();
    let out = serde_json::json!({
        "school_id": school_id.as_uuid().to_string(),
        "student_id": student_id.to_string(),
        "first_name": first,
        "last_name": last,
        "class_id": class_id.to_string(),
        "section_id": section_id.to_string(),
        "correlation_id": corr.as_uuid().to_string(),
        "admitted_by": user.as_uuid().to_string(),
    });
    tracing::info!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}

/// Mark a bulk attendance row. Prints the row id as JSON.
pub async fn attendance(school: String, student: String, date: String, status: String) -> Result<()> {
    let world = test_world();
    let school_id = parse_school(&school)?;
    let student_id = parse_uuid(&student, "student")?;
    let g = SystemIdGen;
    let user = g.next_user_id();

    let date = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| anyhow!("invalid date {date:?}: {e}"))?;
    let attendance_type = match status.as_str() {
        "P" | "present" => "P",
        "A" | "absent" => "A",
        "L" | "late" => "L",
        "F" | "leave" => "F",
        "H" | "holiday" => "H",
        _ => return Err(anyhow!("invalid status {status:?}; expected P|A|L|F|H")),
    };
    let is_absent = matches!(attendance_type, "A");

    let row = StudentAttendanceRow {
        school_id,
        id: g.next_uuid(),
        student_id,
        student_record_id: g.next_uuid(),
        class_id: g.next_uuid(),
        section_id: g.next_uuid(),
        attendance_date: date,
        attendance_type: attendance_type.to_owned(),
        in_time: None,
        out_time: None,
        notes: None,
        is_absent,
        marked_by: user,
        marked_at: Timestamp::now(),
        marked_from: "manual".to_owned(),
        version: Version::initial(),
        etag: Etag::new("00000000000000000000000000000001")?,
        created_at: Timestamp::now(),
        updated_at: Timestamp::now(),
        created_by: user,
        updated_by: user,
        active_status: ActiveStatus::Active,
        correlation_id: g.next_correlation_id(),
        last_event_id: Some(g.next_event_id()),
    };

    let ctx = TenantContext::for_user(
        school_id,
        user,
        g.next_correlation_id(),
        UserType::SchoolAdmin,
    );
    world
        .storage
        .bulk_insert_student_attendances(&ctx, std::slice::from_ref(&row))
        .await
        .map_err(|e| anyhow!("attendance insert failed: {e}"))?;

    let out = serde_json::json!({
        "row_id": row.id.to_string(),
        "school_id": school_id.as_uuid().to_string(),
        "student_id": student_id.to_string(),
        "attendance_date": date.to_string(),
        "attendance_type": attendance_type,
        "is_absent": is_absent,
    });
    tracing::info!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}

/// Record a payment. Prints the receipt id as JSON.
#[allow(clippy::too_many_arguments)]
pub async fn payment(
    school: String,
    invoice: String,
    amount: i64,
    currency: String,
    method: String,
) -> Result<()> {
    let world = test_world();
    let school_id = parse_school(&school)?;
    let g = SystemIdGen;
    let user = g.next_user_id();
    let ctx = TenantContext::for_user(
        school_id,
        user,
        g.next_correlation_id(),
        UserType::SchoolAdmin,
    );
    let currency_code = CurrencyCode::new(&currency)
        .map_err(|e| anyhow!("invalid currency {currency:?}: {e:?}"))?;
    let money = Money::new(currency_code, amount)
        .map_err(|e| anyhow!("invalid amount {amount}: {e:?}"))?;
    let payment_method = match method.as_str() {
        "cash" => PaymentMethod::Cash,
        "card" => PaymentMethod::Card {
            token: CardToken::new("tok-test"),
            save: false,
        },
        "cheque" => PaymentMethod::Cheque {
            number: "000123".to_owned(),
            bank: "HDFC".to_owned(),
            date: ChequeDate::new(2026, 6, 21)?,
        },
        _ => return Err(anyhow!("invalid method {method:?}; expected cash|card|cheque")),
    };
    let req = ChargeRequest::new(
        ctx,
        money,
        payment_method,
        CustomerRef::External(CustomerId::new(invoice)),
        g.next_idempotency_key(),
        g.next_correlation_id(),
    );
    let receipt = world
        .payment
        .charge(req)
        .await
        .map_err(|e| anyhow!("payment charge failed: {e:?}"))?;
    let out = serde_json::json!({
        "payment_id": receipt.payment_id.as_str(),
        "amount_minor": receipt.amount.amount_minor,
        "currency": receipt.amount.currency.as_str(),
        "method": receipt.method.to_string(),
        "status": format!("{:?}", receipt.status),
    });
    tracing::info!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}

fn parse_school(s: &str) -> Result<SchoolId> {
    let u = parse_uuid(s, "school")?;
    Ok(SchoolId::from_uuid(u))
}

fn parse_uuid(s: &str, name: &str) -> Result<Uuid> {
    Uuid::from_str(s).map_err(|e| anyhow!("invalid {name} uuid {s:?}: {e}"))
}

/// Dispatch a parsed `Command` to the right handler.
pub async fn dispatch(cmd: Command) -> Result<()> {
    match cmd {
        Command::Admit {
            school,
            first,
            last,
            class,
            section,
        } => admit(school, first, last, class, section).await,
        Command::Attendance {
            school,
            student,
            date,
            status,
        } => attendance(school, student, date, status).await,
        Command::Payment {
            school,
            invoice,
            amount,
            currency,
            method,
        } => payment(school, invoice, amount, currency, method).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cli;
    use clap::Parser;

    #[test]
    fn parse_admit_args() {
        let cli = Cli::try_parse_from([
            "educore-cli",
            "admit",
            "--school",
            "00000000-0000-0000-0000-000000000001",
            "--first",
            "Ada",
            "--last",
            "Lovelace",
            "--class",
            "00000000-0000-0000-0000-000000000002",
            "--section",
            "00000000-0000-0000-0000-000000000003",
        ])
        .unwrap();
        match cli.command {
            Command::Admit {
                school,
                first,
                last,
                class,
                section,
            } => {
                assert_eq!(school, "00000000-0000-0000-0000-000000000001");
                assert_eq!(first, "Ada");
                assert_eq!(last, "Lovelace");
                assert_eq!(class, "00000000-0000-0000-0000-000000000002");
                assert_eq!(section, "00000000-0000-0000-0000-000000000003");
            }
            _ => panic!("expected Admit"),
        }
    }

    #[test]
    fn parse_attendance_args() {
        let cli = Cli::try_parse_from([
            "educore-cli",
            "attendance",
            "--school",
            "00000000-0000-0000-0000-000000000001",
            "--student",
            "00000000-0000-0000-0000-000000000004",
            "--date",
            "2026-06-21",
            "--status",
            "P",
        ])
        .unwrap();
        match cli.command {
            Command::Attendance {
                school,
                student,
                date,
                status,
            } => {
                assert_eq!(school, "00000000-0000-0000-0000-000000000001");
                assert_eq!(student, "00000000-0000-0000-0000-000000000004");
                assert_eq!(date, "2026-06-21");
                assert_eq!(status, "P");
            }
            _ => panic!("expected Attendance"),
        }
    }

    #[test]
    fn parse_payment_args() {
        let cli = Cli::try_parse_from([
            "educore-cli",
            "payment",
            "--school",
            "00000000-0000-0000-0000-000000000001",
            "--invoice",
            "00000000-0000-0000-0000-000000000005",
            "--amount",
            "1500",
            "--currency",
            "USD",
            "--method",
            "cash",
        ])
        .unwrap();
        match cli.command {
            Command::Payment {
                school,
                invoice,
                amount,
                currency,
                method,
            } => {
                assert_eq!(school, "00000000-0000-0000-0000-000000000001");
                assert_eq!(amount, 1500);
                assert_eq!(currency, "USD");
                assert_eq!(method, "cash");
                assert_eq!(invoice, "00000000-0000-0000-0000-000000000005");
            }
            _ => panic!("expected Payment"),
        }
    }
}
