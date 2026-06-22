use std::collections::HashMap;

use chrono::{Duration, NaiveDateTime, Utc};
use loco_rs::{Error, Result};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, QueryFilter, QueryOrder, TransactionTrait, TryIntoModel,
};

use crate::models::{
    _entities::{groups, members, transactions, users},
    users::{Model as User, RegisterParams},
};

const DEMO_PREFIX: &str = "DEMO-";

struct GroupSeed {
    code: &'static str,
    name: &'static str,
    description: &'static str,
    contribution_amount: i64,
    frequency: &'static str,
    payout_offset_days: i64,
}

const GROUPS: [GroupSeed; 4] = [
    GroupSeed {
        code: "MKL",
        name: "Makola Market Circle",
        description: "Market traders building reliable working capital together",
        contribution_amount: 250,
        frequency: "weekly",
        payout_offset_days: 5,
    },
    GroupSeed {
        code: "AOS",
        name: "Abossey Okai Traders",
        description: "Automotive parts traders supporting business growth",
        contribution_amount: 500,
        frequency: "monthly",
        payout_offset_days: 12,
    },
    GroupSeed {
        code: "KNT",
        name: "Kaneshie Traders Union",
        description: "Kaneshie traders pooling funds for stock and emergencies",
        contribution_amount: 150,
        frequency: "weekly",
        payout_offset_days: 2,
    },
    GroupSeed {
        code: "TMA",
        name: "Tema Community Builders",
        description: "Community entrepreneurs financing durable local businesses",
        contribution_amount: 800,
        frequency: "monthly",
        payout_offset_days: 18,
    },
];

struct MemberSeed {
    group_code: &'static str,
    name: &'static str,
    phone: &'static str,
    trust_score: i32,
    total_contributed: i64,
    on_time_payments: i32,
    total_payments: i32,
}

const MEMBERS: [MemberSeed; 12] = [
    MemberSeed {
        group_code: "MKL",
        name: "Akosua Mensah",
        phone: "+233244105201",
        trust_score: 96,
        total_contributed: 5_250,
        on_time_payments: 21,
        total_payments: 22,
    },
    MemberSeed {
        group_code: "MKL",
        name: "Kwame Asare",
        phone: "+233205774318",
        trust_score: 88,
        total_contributed: 4_750,
        on_time_payments: 19,
        total_payments: 21,
    },
    MemberSeed {
        group_code: "MKL",
        name: "Esi Badu",
        phone: "+233557903842",
        trust_score: 92,
        total_contributed: 5_000,
        on_time_payments: 20,
        total_payments: 21,
    },
    MemberSeed {
        group_code: "AOS",
        name: "Kofi Ofori",
        phone: "+233243811506",
        trust_score: 98,
        total_contributed: 6_500,
        on_time_payments: 13,
        total_payments: 13,
    },
    MemberSeed {
        group_code: "AOS",
        name: "Adwoa Nyarko",
        phone: "+233506624917",
        trust_score: 84,
        total_contributed: 5_500,
        on_time_payments: 11,
        total_payments: 13,
    },
    MemberSeed {
        group_code: "AOS",
        name: "Yaw Frimpong",
        phone: "+233277315690",
        trust_score: 78,
        total_contributed: 4_500,
        on_time_payments: 9,
        total_payments: 12,
    },
    MemberSeed {
        group_code: "KNT",
        name: "Ama Serwaa",
        phone: "+233541228304",
        trust_score: 94,
        total_contributed: 3_300,
        on_time_payments: 22,
        total_payments: 23,
    },
    MemberSeed {
        group_code: "KNT",
        name: "Kojo Ansah",
        phone: "+233208491735",
        trust_score: 81,
        total_contributed: 2_850,
        on_time_payments: 19,
        total_payments: 22,
    },
    MemberSeed {
        group_code: "KNT",
        name: "Abena Owusu",
        phone: "+233594006218",
        trust_score: 68,
        total_contributed: 2_400,
        on_time_payments: 16,
        total_payments: 21,
    },
    MemberSeed {
        group_code: "TMA",
        name: "Nana Yeboah",
        phone: "+233245519873",
        trust_score: 91,
        total_contributed: 7_200,
        on_time_payments: 9,
        total_payments: 10,
    },
    MemberSeed {
        group_code: "TMA",
        name: "Efua Sarpong",
        phone: "+233501837426",
        trust_score: 86,
        total_contributed: 6_400,
        on_time_payments: 8,
        total_payments: 10,
    },
    MemberSeed {
        group_code: "TMA",
        name: "Daniel Tetteh",
        phone: "+233273662105",
        trust_score: 73,
        total_contributed: 4_800,
        on_time_payments: 6,
        total_payments: 9,
    },
];

struct TransactionSeed {
    reference: &'static str,
    member_phone: &'static str,
    group_code: &'static str,
    amount: i64,
    kind: &'static str,
    status: &'static str,
    days_ago: i64,
}

const TRANSACTIONS: [TransactionSeed; 20] = [
    TransactionSeed {
        reference: "DEMO-MKL-001",
        member_phone: "+233244105201",
        group_code: "MKL",
        amount: 250,
        kind: "contribution",
        status: "completed",
        days_ago: 2,
    },
    TransactionSeed {
        reference: "DEMO-MKL-002",
        member_phone: "+233205774318",
        group_code: "MKL",
        amount: 250,
        kind: "contribution",
        status: "pending",
        days_ago: 3,
    },
    TransactionSeed {
        reference: "DEMO-MKL-003",
        member_phone: "+233557903842",
        group_code: "MKL",
        amount: 250,
        kind: "contribution",
        status: "completed",
        days_ago: 9,
    },
    TransactionSeed {
        reference: "DEMO-MKL-004",
        member_phone: "+233244105201",
        group_code: "MKL",
        amount: 3_000,
        kind: "payout",
        status: "completed",
        days_ago: 18,
    },
    TransactionSeed {
        reference: "DEMO-MKL-005",
        member_phone: "+233205774318",
        group_code: "MKL",
        amount: 250,
        kind: "contribution",
        status: "completed",
        days_ago: 41,
    },
    TransactionSeed {
        reference: "DEMO-AOS-001",
        member_phone: "+233243811506",
        group_code: "AOS",
        amount: 500,
        kind: "contribution",
        status: "completed",
        days_ago: 5,
    },
    TransactionSeed {
        reference: "DEMO-AOS-002",
        member_phone: "+233506624917",
        group_code: "AOS",
        amount: 500,
        kind: "contribution",
        status: "completed",
        days_ago: 16,
    },
    TransactionSeed {
        reference: "DEMO-AOS-003",
        member_phone: "+233277315690",
        group_code: "AOS",
        amount: 500,
        kind: "contribution",
        status: "pending",
        days_ago: 28,
    },
    TransactionSeed {
        reference: "DEMO-AOS-004",
        member_phone: "+233243811506",
        group_code: "AOS",
        amount: 5_500,
        kind: "payout",
        status: "completed",
        days_ago: 67,
    },
    TransactionSeed {
        reference: "DEMO-AOS-005",
        member_phone: "+233506624917",
        group_code: "AOS",
        amount: 500,
        kind: "contribution",
        status: "completed",
        days_ago: 96,
    },
    TransactionSeed {
        reference: "DEMO-KNT-001",
        member_phone: "+233541228304",
        group_code: "KNT",
        amount: 150,
        kind: "contribution",
        status: "completed",
        days_ago: 1,
    },
    TransactionSeed {
        reference: "DEMO-KNT-002",
        member_phone: "+233208491735",
        group_code: "KNT",
        amount: 150,
        kind: "contribution",
        status: "completed",
        days_ago: 12,
    },
    TransactionSeed {
        reference: "DEMO-KNT-003",
        member_phone: "+233594006218",
        group_code: "KNT",
        amount: 150,
        kind: "contribution",
        status: "failed",
        days_ago: 34,
    },
    TransactionSeed {
        reference: "DEMO-KNT-004",
        member_phone: "+233541228304",
        group_code: "KNT",
        amount: 1_800,
        kind: "payout",
        status: "completed",
        days_ago: 73,
    },
    TransactionSeed {
        reference: "DEMO-KNT-005",
        member_phone: "+233208491735",
        group_code: "KNT",
        amount: 150,
        kind: "contribution",
        status: "completed",
        days_ago: 128,
    },
    TransactionSeed {
        reference: "DEMO-TMA-001",
        member_phone: "+233245519873",
        group_code: "TMA",
        amount: 800,
        kind: "contribution",
        status: "completed",
        days_ago: 7,
    },
    TransactionSeed {
        reference: "DEMO-TMA-002",
        member_phone: "+233501837426",
        group_code: "TMA",
        amount: 800,
        kind: "contribution",
        status: "pending",
        days_ago: 22,
    },
    TransactionSeed {
        reference: "DEMO-TMA-003",
        member_phone: "+233273662105",
        group_code: "TMA",
        amount: 800,
        kind: "contribution",
        status: "completed",
        days_ago: 52,
    },
    TransactionSeed {
        reference: "DEMO-TMA-004",
        member_phone: "+233245519873",
        group_code: "TMA",
        amount: 7_200,
        kind: "payout",
        status: "completed",
        days_ago: 101,
    },
    TransactionSeed {
        reference: "DEMO-TMA-005",
        member_phone: "+233501837426",
        group_code: "TMA",
        amount: 800,
        kind: "contribution",
        status: "completed",
        days_ago: 157,
    },
];

/// Populate the non-production demonstration dataset without touching unrelated rows.
pub async fn run(db: &DatabaseConnection) -> Result<()> {
    let today = seed_anchor();
    let admin_exists = validate_demo_admin(db).await?;

    let transaction = db.begin().await?;
    if !admin_exists {
        User::create_with_password_in(&transaction, &demo_admin_params()).await?;
    }

    let mut group_models = HashMap::new();
    for seed in &GROUPS {
        let model = upsert_group(&transaction, seed, today).await?;
        group_models.insert(seed.code, model);
    }

    let mut member_models = HashMap::new();
    for seed in &MEMBERS {
        let group = group_for(&group_models, seed.group_code)?;
        let model = upsert_member(&transaction, seed, group.id, today).await?;
        member_models.insert((seed.group_code, seed.phone), model);
    }

    recompute_group_totals(&transaction, &mut group_models).await?;
    replace_demo_transactions(&transaction, today, &group_models, &member_models).await?;
    transaction.commit().await?;

    Ok(())
}

/// Anchor relative demo dates at midnight UTC so reruns within a seed day are
/// identical while running the seed on a later day intentionally refreshes recency.
fn seed_anchor() -> NaiveDateTime {
    Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("midnight is a valid time")
}

/// Compatibility entry point for existing seed callers.
pub async fn seed(db: &DatabaseConnection) -> Result<()> {
    run(db).await
}

fn demo_admin_params() -> RegisterParams {
    RegisterParams {
        email: "admin@trustledger.gh".to_owned(),
        password: "Demo@2026".to_owned(),
        name: "TrustLedger Demo Admin".to_owned(),
    }
}

async fn validate_demo_admin(db: &DatabaseConnection) -> Result<bool> {
    let existing = users::Entity::find()
        .filter(users::Column::Email.eq("admin@trustledger.gh"))
        .one(db)
        .await?;

    if let Some(user) = existing {
        if !user.verify_password("Demo@2026") {
            return Err(Error::Message(
                "demo admin admin@trustledger.gh exists with credentials not owned by the demo seed"
                    .to_owned(),
            ));
        }
        return Ok(true);
    }

    Ok(false)
}

fn group_for<'a>(
    groups_by_code: &'a HashMap<&'static str, groups::Model>,
    code: &str,
) -> Result<&'a groups::Model> {
    groups_by_code
        .get(code)
        .ok_or_else(|| Error::Message(format!("demo seed group code {code} was not loaded")))
}

fn member_for<'a>(
    members_by_key: &'a HashMap<(&'static str, &'static str), members::Model>,
    group_code: &'static str,
    phone: &'static str,
) -> Result<&'a members::Model> {
    members_by_key.get(&(group_code, phone)).ok_or_else(|| {
        Error::Message(format!(
            "demo seed member {phone} in group {group_code} was not loaded"
        ))
    })
}

async fn upsert_group<C: ConnectionTrait>(
    db: &C,
    seed: &GroupSeed,
    today: NaiveDateTime,
) -> Result<groups::Model> {
    let existing = groups::Entity::find()
        .filter(groups::Column::Code.eq(seed.code))
        .one(db)
        .await?;
    let mut active: groups::ActiveModel = existing.map(Into::into).unwrap_or_default();
    active.name = Set(seed.name.to_owned());
    active.code = Set(seed.code.to_owned());
    active.description = Set(Some(seed.description.to_owned()));
    active.contribution_amount = Set(Some(seed.contribution_amount.into()));
    active.frequency = Set(seed.frequency.to_owned());
    active.next_payout_date = Set(today + Duration::days(seed.payout_offset_days));
    active.is_active = Set(Some(true));
    active.created_at = Set(Some(today - Duration::days(365)));
    Ok(active.save(db).await?.try_into_model()?)
}

async fn upsert_member<C: ConnectionTrait>(
    db: &C,
    seed: &MemberSeed,
    group_id: i32,
    today: NaiveDateTime,
) -> Result<members::Model> {
    let existing = members::Entity::find()
        .filter(members::Column::Phone.eq(seed.phone))
        .filter(members::Column::GroupId.eq(group_id))
        .order_by_asc(members::Column::Id)
        .all(db)
        .await?;
    if existing.len() > 1 {
        return Err(Error::Message(format!(
            "demo seed found {} members for duplicate key (group_id: {group_id}, phone: {})",
            existing.len(),
            seed.phone
        )));
    }
    let mut active: members::ActiveModel = existing
        .into_iter()
        .next()
        .map(Into::into)
        .unwrap_or_default();
    active.name = Set(seed.name.to_owned());
    active.phone = Set(seed.phone.to_owned());
    active.trust_score = Set(Some(seed.trust_score));
    active.group_id = Set(group_id);
    active.total_contributed = Set(Some(seed.total_contributed.into()));
    active.on_time_payments = Set(Some(seed.on_time_payments));
    active.total_payments = Set(Some(seed.total_payments));
    active.last_missed_payment = if seed.on_time_payments == seed.total_payments {
        Set(None)
    } else {
        Set(Some(today - Duration::days(30)))
    };
    active.status = Set(Some("active".to_owned()));
    active.joined_at = Set(Some(today - Duration::days(300)));
    Ok(active.save(db).await?.try_into_model()?)
}

async fn recompute_group_totals<C: ConnectionTrait>(
    db: &C,
    groups_by_code: &mut HashMap<&'static str, groups::Model>,
) -> Result<()> {
    for group in groups_by_code.values_mut() {
        let group_members = members::Entity::find()
            .filter(members::Column::GroupId.eq(group.id))
            .all(db)
            .await?;
        let total = group_members
            .iter()
            .fold(sea_orm::prelude::Decimal::ZERO, |sum, member| {
                sum + member.total_contributed.unwrap_or_default()
            });
        let mut active: groups::ActiveModel = group.clone().into();
        active.member_count = Set(Some(group_members.len() as i32));
        active.total_contributed = Set(Some(total));
        *group = active.update(db).await?;
    }
    Ok(())
}

async fn replace_demo_transactions<C: ConnectionTrait>(
    db: &C,
    today: NaiveDateTime,
    groups_by_code: &HashMap<&'static str, groups::Model>,
    members_by_key: &HashMap<(&'static str, &'static str), members::Model>,
) -> Result<()> {
    let existing = transactions::Entity::find()
        .filter(transactions::Column::MoolreRef.starts_with(DEMO_PREFIX))
        .order_by_asc(transactions::Column::Id)
        .all(db)
        .await?;
    let preserved_ids: HashMap<String, i32> = existing
        .iter()
        .filter_map(|transaction| {
            transaction
                .moolre_ref
                .clone()
                .map(|reference| (reference, transaction.id))
        })
        .collect();

    transactions::Entity::delete_many()
        .filter(transactions::Column::MoolreRef.starts_with(DEMO_PREFIX))
        .exec(db)
        .await?;

    for seed in &TRANSACTIONS {
        let group = group_for(groups_by_code, seed.group_code)?;
        let member = member_for(members_by_key, seed.group_code, seed.member_phone)?;
        let mut active = transactions::ActiveModel {
            member_id: Set(member.id),
            member_name: Set(member.name.clone()),
            group_id: Set(group.id),
            group_name: Set(group.name.clone()),
            amount: Set(seed.amount.into()),
            r#type: Set(seed.kind.to_owned()),
            status: Set(seed.status.to_owned()),
            moolre_ref: Set(Some(seed.reference.to_owned())),
            date: Set(Some(today - Duration::days(seed.days_ago))),
            ..Default::default()
        };
        if let Some(id) = preserved_ids.get(seed.reference) {
            active.id = Set(*id);
        }
        active.insert(db).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::Timelike;

    use super::{group_for, seed_anchor};

    #[test]
    fn seed_anchor_is_the_current_utc_day_boundary() {
        let anchor = seed_anchor();

        assert_eq!((anchor.hour(), anchor.minute(), anchor.second()), (0, 0, 0));
    }

    #[test]
    fn missing_group_lookup_returns_a_descriptive_error() {
        let groups = HashMap::new();

        let error = group_for(&groups, "UNKNOWN").expect_err("missing group should fail");

        assert!(error.to_string().contains("UNKNOWN"));
    }
}
