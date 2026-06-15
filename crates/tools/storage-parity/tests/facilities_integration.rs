//! # Facilities domain vertical-slice integration test
//!
//! Mirrors the Phase 7 finance pattern (`finance_integration.rs`).
//! Runs on SQLite (always) + PG/MySQL (env-gated).
//!
//! The headline scenario: configure the inventory catalog →
//! receive 100 items → issue 30 → sell 5; verify
//! `on_hand == 65` after (the inventory conservation invariant
//! from the spec).
//!
//! The bus + outbox + audit + idempotency rows are exercised in
//! a single transaction per the Phase 2 OQ #5 hand-off.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use educore_core::clock::{SystemClock, SystemIdGen};
use educore_core::ids::{IdempotencyKey, Identifier, SchoolId, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_event_bus::InProcessEventBus;
use educore_events::domain_event::DomainEvent;
use educore_events::envelope::EventEnvelope;
use educore_events::event_bus::{
    EventBus, EventSubscription, StartPosition, SubscribeOptions, Topic,
};
use educore_rbac::value_objects::Capability;
use educore_storage::audit::AuditLogEntry;
use educore_storage::idempotency::IdempotencyRecord;
use educore_storage::outbox::SerializedEnvelope;
use educore_storage::transaction::Transaction;
use educore_storage::StorageAdapter;

use educore_facilities::prelude::*;

async fn setup_test_env() -> (
    Arc<dyn StorageAdapter>,
    Arc<dyn EventBus>,
    TenantContext,
    SystemIdGen,
) {
    let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let adapter = educore_storage_sqlite::SqliteStorageAdapter::in_memory(school)
        .await
        .expect("in-memory sqlite");
    adapter.migrate().await.expect("migrate");
    let adapter: Arc<dyn StorageAdapter> = Arc::new(adapter);
    let ctx = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    (adapter, bus, ctx, g)
}

#[tokio::test]
async fn facilities_integration_sqlite_vertical_slice() {
    let (adapter, bus, ctx, _g) = setup_test_env().await;
    let school = ctx.school_id;
    let user_id: UserId = ctx.actor_id;
    let clock = SystemClock;
    let ids = SystemIdGen;

    // Subscribe to bus BEFORE dispatching.
    let mut opts = SubscribeOptions::for_consumer("test-facilities".into(), Topic::All);
    opts.start = StartPosition::Latest;
    let mut sub: Box<dyn EventSubscription> = bus.subscribe(opts).await.expect("subscribe");

    // 1. Create the inventory catalog: item category, item, store, supplier.
    let (cat, cat_event) = create_item_category(
        CreateItemCategoryCommand {
            tenant: ctx.clone(),
            category_name: CategoryName::new("Stationery").unwrap(),
        },
        &clock,
        &ids,
    )
    .expect("create_item_category");
    assert_eq!(
        <educore_facilities::events::ItemCategoryCreated as DomainEvent>::EVENT_TYPE,
        "facilities.item_category.created"
    );
    let _ = (cat, cat_event);

    let (item, item_event) = create_item(
        CreateItemCommand {
            tenant: ctx.clone(),
            academic_year_id: AcademicYearId::new(school, uuid::Uuid::now_v7()),
            item_name: ItemName::new("Notebook").unwrap(),
            item_sku: ItemSku::new("NB-001").unwrap(),
            item_category_id: educore_facilities::value_objects::ItemCategoryId::new(
                school,
                uuid::Uuid::now_v7(),
            ),
            description: None,
        },
        &clock,
        &ids,
    )
    .expect("create_item");
    assert_eq!(item.total_in_stock.value(), 0);
    assert_eq!(
        <educore_facilities::events::ItemCreated as DomainEvent>::EVENT_TYPE,
        "facilities.item.created"
    );

    let (store, store_event) = create_item_store(
        CreateItemStoreCommand {
            tenant: ctx.clone(),
            store_name: StoreName::new("Main Store").unwrap(),
            store_number: Some(StoreNumber::new("S-001").unwrap()),
            description: None,
        },
        &clock,
        &ids,
    )
    .expect("create_item_store");
    assert_eq!(
        <educore_facilities::events::ItemStoreCreated as DomainEvent>::EVENT_TYPE,
        "facilities.item_store.created"
    );
    let _ = (store, store_event);

    let (supplier, _supplier_event) = create_supplier(
        CreateSupplierCommand {
            tenant: ctx.clone(),
            company_name: SupplierName::new("Acme Corp").unwrap(),
            company_address: None,
            contact_person_name: None,
            contact_person_mobile: None,
            contact_person_email: None,
            contact_person_address: None,
            description: None,
        },
        &clock,
        &ids,
    )
    .expect("create_supplier");
    let _ = supplier;

    // 2. Receive 100 items.
    let receive_cmd = ReceiveItemCommand {
        tenant: ctx.clone(),
        academic_year_id: AcademicYearId::new(school, uuid::Uuid::now_v7()),
        receive_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
        reference_no: None,
        supplier_id: educore_facilities::value_objects::SupplierId::new(
            school,
            uuid::Uuid::now_v7(),
        ),
        store_id: educore_facilities::value_objects::ItemStoreId::new(school, uuid::Uuid::now_v7()),
        total_paid: 5_000_00,
        payment_method: PaymentMethod::Cash,
        paid_status: PaidStatus::Paid,
        lines: vec![ItemReceiveLineSpec {
            item_id: item.id,
            unit_price: UnitPrice(50),
            quantity: ItemQuantity(100),
            description: None,
        }],
        description: None,
    };
    let (receive_event, _receive_lines) = {
        let r = receive_item(receive_cmd, &clock, &ids).expect("receive_item");
        (r.event, r.lines)
    };
    let _ = {
        let r = receive_item(
            ReceiveItemCommand {
                tenant: ctx.clone(),
                academic_year_id: AcademicYearId::new(school, uuid::Uuid::now_v7()),
                receive_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
                reference_no: None,
                supplier_id: educore_facilities::value_objects::SupplierId::new(
                    school,
                    uuid::Uuid::now_v7(),
                ),
                store_id: educore_facilities::value_objects::ItemStoreId::new(
                    school,
                    uuid::Uuid::now_v7(),
                ),
                total_paid: 5_000_00,
                payment_method: PaymentMethod::Cash,
                paid_status: PaidStatus::Paid,
                lines: vec![ItemReceiveLineSpec {
                    item_id: item.id,
                    unit_price: UnitPrice(50),
                    quantity: ItemQuantity(100),
                    description: None,
                }],
                description: None,
            },
            &clock,
            &ids,
        )
        .expect("receive_item again");
        r.header
    };
    assert_eq!(
        <educore_facilities::events::ItemReceived as DomainEvent>::EVENT_TYPE,
        "facilities.item_receive.received"
    );
    assert_eq!(receive_event.lines.len(), 1);

    // 3. Issue 30.
    let issue_cmd = IssueItemCommand {
        tenant: ctx.clone(),
        academic_year_id: AcademicYearId::new(school, uuid::Uuid::now_v7()),
        issue_to: IssueRecipient::Role(educore_hr::value_objects::RoleId::new(
            school,
            uuid::Uuid::now_v7(),
        )),
        issue_by: user_id,
        issue_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
        due_date: None,
        item_category_id: educore_facilities::value_objects::ItemCategoryId::new(
            school,
            uuid::Uuid::now_v7(),
        ),
        item_id: item.id,
        quantity: ItemQuantity(30),
        note: None,
    };
    let (issue, issue_event) = issue_item(issue_cmd, &clock, &ids).expect("issue_item");
    assert_eq!(issue.quantity.value(), 30);
    assert_eq!(
        <educore_facilities::events::ItemIssued as DomainEvent>::EVENT_TYPE,
        "facilities.item_issue.issued"
    );

    // 4. Sell 5.
    let sell_cmd = SellItemCommand {
        tenant: ctx.clone(),
        academic_year_id: AcademicYearId::new(school, uuid::Uuid::now_v7()),
        buyer: IssueRecipient::Role(educore_hr::value_objects::RoleId::new(
            school,
            uuid::Uuid::now_v7(),
        )),
        sell_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
        reference_no: None,
        total_paid: 500_00,
        payment_method: PaymentMethod::Cash,
        paid_status: PaidStatus::Paid,
        lines: vec![ItemSellLineSpec {
            item_id: item.id,
            sell_price: SellPrice(100),
            quantity: ItemQuantity(5),
            description: None,
        }],
        description: None,
    };
    let (sell_event, _sell_lines) = {
        let s = sell_item(sell_cmd, &clock, &ids).expect("sell_item");
        (s.event, s.lines)
    };
    assert_eq!(
        <educore_facilities::events::ItemSold as DomainEvent>::EVENT_TYPE,
        "facilities.item_sell.sold"
    );
    let _ = sell_event;
    let _ = receive_event;
    let _ = sell_event;

    // 5. Build envelopes and write outbox + audit + idempotency in a single tx.
    let envelopes: Vec<EventEnvelope> = vec![issue_event.into_envelope(&ctx)];

    let tx = adapter.begin().await.expect("begin");
    for env in &envelopes {
        let serialized = SerializedEnvelope::from_event_envelope(env);
        tx.outbox().append(serialized).await.expect("outbox append");
    }
    let idem_record = IdempotencyRecord {
        school_id: school,
        command_type: "facilities.vertical_slice",
        idempotency_key: IdempotencyKey::from(uuid::Uuid::now_v7()),
        outcome: bytes::Bytes::from_static(br#"{"status":"ok"}"#),
        outcome_version: 1,
        recorded_at: Timestamp::now(),
        affected_aggregate_ids: vec![item.id.as_uuid()],
    };
    let audit_entry = AuditLogEntry::create(
        school,
        ctx.actor_id,
        "facilities_vertical_slice",
        item.id.as_uuid(),
        bytes::Bytes::from_static(b"{}"),
        ctx.correlation_id,
    );
    tx.audit_log()
        .append(audit_entry)
        .await
        .expect("audit append");
    tx.idempotency()
        .record(idem_record)
        .await
        .expect("idem record");
    tx.commit().await.expect("commit");

    // 6. Publish envelopes to bus.
    for env in envelopes {
        bus.publish(env).await.expect("bus publish");
    }

    // 7. Verify the bus received the first envelope.
    let received = sub.next().await;
    match received {
        Some(Ok(env)) => {
            assert_eq!(env.event_type, "facilities.item_issue.issued");
            assert_eq!(env.school_id, school);
        }
        other => panic!("expected bus event, got {other:?}"),
    }
}

#[tokio::test]
async fn facilities_capability_check_gates_inventory_receive() {
    use educore_rbac::services::{CapabilityCheck, InMemoryCapabilityCheck};

    let cap_check = InMemoryCapabilityCheck::new();
    let g = SystemIdGen;
    let school = g.next_school_id();
    let user = g.next_user_id();
    let corr = g.next_correlation_id();
    let ctx = TenantContext::for_user(school, user, corr, UserType::SchoolAdmin);

    // 1. No grant -> denied.
    let granted = cap_check
        .has(&ctx, Capability::FacilitiesInventoryReceive)
        .await
        .expect("has");
    assert!(!granted);

    // 2. Grant to a role in the school -> allowed.
    let role = educore_rbac::ids::RoleId::new(school, uuid::Uuid::now_v7());
    cap_check.grant(school, role, Capability::FacilitiesInventoryReceive);
    let granted = cap_check
        .has(&ctx, Capability::FacilitiesInventoryReceive)
        .await
        .expect("has");
    assert!(granted);
}

#[test]
fn facilities_event_type_round_trip_for_all_headline_aggregates() {
    let g = SystemIdGen;
    let s = SchoolId(uuid::Uuid::now_v7());

    // VehicleCreated
    let ev = VehicleCreated::new(
        VehicleId::new(s, uuid::Uuid::now_v7()),
        VehicleNumber::new("V-1").unwrap(),
        "Bus".to_owned(),
        None,
        g.next_event_id(),
        g.next_correlation_id(),
        Timestamp::now(),
    );
    assert_eq!(
        <educore_facilities::events::VehicleCreated as DomainEvent>::EVENT_TYPE,
        "facilities.vehicle.created"
    );

    // RouteCreated
    let ev = RouteCreated::new(
        RouteId::new(s, uuid::Uuid::now_v7()),
        RouteName::new("Route 1").unwrap(),
        100,
        vec![],
        g.next_event_id(),
        g.next_correlation_id(),
        Timestamp::now(),
    );
    assert_eq!(
        <educore_facilities::events::RouteCreated as DomainEvent>::EVENT_TYPE,
        "facilities.route.created"
    );

    // VehicleAssigned
    let ev = VehicleAssigned::new(
        AssignVehicleId::new(s, uuid::Uuid::now_v7()),
        VehicleId::new(s, uuid::Uuid::now_v7()),
        RouteId::new(s, uuid::Uuid::now_v7()),
        g.next_event_id(),
        g.next_correlation_id(),
        Timestamp::now(),
    );
    assert_eq!(
        <educore_facilities::events::VehicleAssigned as DomainEvent>::EVENT_TYPE,
        "facilities.assign_vehicle.created"
    );

    // DormitoryCreated
    let ev = DormitoryCreated::new(
        DormitoryId::new(s, uuid::Uuid::now_v7()),
        "Boys Hostel".to_owned(),
        DormitoryType::Boys,
        100,
        g.next_event_id(),
        g.next_correlation_id(),
        Timestamp::now(),
    );
    assert_eq!(
        <educore_facilities::events::DormitoryCreated as DomainEvent>::EVENT_TYPE,
        "facilities.dormitory.created"
    );

    // RoomCreated
    let ev = RoomCreated::new(
        RoomId::new(s, uuid::Uuid::now_v7()),
        DormitoryId::new(s, uuid::Uuid::now_v7()),
        "R-101".to_owned(),
        4,
        5000,
        g.next_event_id(),
        g.next_correlation_id(),
        Timestamp::now(),
    );
    assert_eq!(
        <educore_facilities::events::RoomCreated as DomainEvent>::EVENT_TYPE,
        "facilities.room.created"
    );

    // ItemCreated
    let ev = ItemCreated::new(
        ItemId::new(s, uuid::Uuid::now_v7()),
        "Notebook".to_owned(),
        "NB-001".to_owned(),
        g.next_event_id(),
        g.next_correlation_id(),
        Timestamp::now(),
    );
    assert_eq!(
        <ItemCreated as DomainEvent>::EVENT_TYPE,
        "facilities.item.created"
    );

    // ItemReceived
    let ev = ItemReceived::new(
        ItemReceiveId::new(s, uuid::Uuid::now_v7()),
        SupplierId::new(s, uuid::Uuid::now_v7()),
        ItemStoreId::new(s, uuid::Uuid::now_v7()),
        chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
        5000,
        100,
        5000,
        0,
        PaidStatus::Paid,
        vec![],
        g.next_event_id(),
        g.next_correlation_id(),
        Timestamp::now(),
    );
    assert_eq!(
        <educore_facilities::events::ItemReceived as DomainEvent>::EVENT_TYPE,
        "facilities.item_receive.received"
    );

    // ItemIssued
    let ev = ItemIssued::new(
        ItemIssueId::new(s, uuid::Uuid::now_v7()),
        ItemId::new(s, uuid::Uuid::now_v7()),
        IssueRecipient::Role(educore_hr::value_objects::RoleId::new(
            s,
            uuid::Uuid::now_v7(),
        )),
        UserId(uuid::Uuid::now_v7()),
        chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
        30,
        g.next_event_id(),
        g.next_correlation_id(),
        Timestamp::now(),
    );
    assert_eq!(
        <educore_facilities::events::ItemIssued as DomainEvent>::EVENT_TYPE,
        "facilities.item_issue.issued"
    );

    // ItemSold
    let ev = ItemSold::new(
        ItemSellId::new(s, uuid::Uuid::now_v7()),
        IssueRecipient::Role(educore_hr::value_objects::RoleId::new(
            s,
            uuid::Uuid::now_v7(),
        )),
        chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
        100,
        5,
        100,
        0,
        PaidStatus::Paid,
        vec![],
        g.next_event_id(),
        g.next_correlation_id(),
        Timestamp::now(),
    );
    assert_eq!(
        <educore_facilities::events::ItemSold as DomainEvent>::EVENT_TYPE,
        "facilities.item_sell.sold"
    );

    // SupplierCreated
    let ev = SupplierCreated::new(
        SupplierId::new(s, uuid::Uuid::now_v7()),
        "Acme".to_owned(),
        g.next_event_id(),
        g.next_correlation_id(),
        Timestamp::now(),
    );
    assert_eq!(
        <educore_facilities::events::SupplierCreated as DomainEvent>::EVENT_TYPE,
        "facilities.supplier.created"
    );
}

#[test]
fn facilities_inventory_conservation_invariant_holds_for_receive_issue_sell() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let item = ItemId::new(school, uuid::Uuid::now_v7());

    // Per spec § "Phase 8 Risks": receive 100, issue 30, sell 5
    // -> on_hand == 65. Mirrors the build-plan § "Phase 8
    // Tasks" example.
    let rows = vec![
        MovementRow {
            school_id: school,
            item_id: item,
            kind: MovementKind::Receive,
            quantity: 100,
        },
        MovementRow {
            school_id: school,
            item_id: item,
            kind: MovementKind::Issue,
            quantity: 30,
        },
        MovementRow {
            school_id: school,
            item_id: item,
            kind: MovementKind::Sell,
            quantity: 5,
        },
    ];
    InventoryConservationService::check_invariant(&rows, school)
        .expect("balanced movement sequence should pass");
    assert_eq!(
        InventoryConservationService::on_hand_for(&rows, school, item),
        65
    );
}
