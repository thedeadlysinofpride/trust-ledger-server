use loco_rs::testing::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ActiveValue::Set, ColumnTrait, EntityTrait,
    PaginatorTrait, QueryFilter,
};
use serial_test::serial;
use trust_ledger_server::{
    app::App,
    data::demo_seed,
    models::_entities::{groups, members, transactions, users},
};

#[tokio::test]
#[serial]
async fn demo_seed_rolls_back_admin_and_ledger_changes_on_validation_failure() {
    request::<App, _, _>(|_request, ctx| async move {
        demo_seed::run(&ctx.db)
            .await
            .expect("baseline demo seed should succeed");

        users::Entity::delete_many()
            .filter(users::Column::Email.eq("admin@trustledger.gh"))
            .exec(&ctx.db)
            .await
            .expect("demo admin should be removable for the rollback scenario");

        let group = groups::Entity::find()
            .filter(groups::Column::Code.eq("MKL"))
            .one(&ctx.db)
            .await
            .expect("group lookup should succeed")
            .expect("demo group should exist");
        let group_id = group.id;
        let mut group_active: groups::ActiveModel = group.into();
        group_active.name = Set("Rollback Sentinel".to_owned());
        group_active
            .update(&ctx.db)
            .await
            .expect("sentinel group update should succeed");

        let member = members::Entity::find()
            .filter(members::Column::GroupId.eq(group_id))
            .filter(members::Column::Phone.eq("+233244105201"))
            .one(&ctx.db)
            .await
            .expect("member lookup should succeed")
            .expect("demo member should exist");
        let mut duplicate: members::ActiveModel = member.into();
        duplicate.id = NotSet;
        duplicate.name = Set("Rollback Duplicate".to_owned());
        duplicate
            .insert(&ctx.db)
            .await
            .expect("duplicate fixture should be inserted");

        let transaction_count = transactions::Entity::find()
            .filter(transactions::Column::MoolreRef.starts_with("DEMO-"))
            .count(&ctx.db)
            .await
            .expect("transaction count should succeed");

        let error = demo_seed::run(&ctx.db)
            .await
            .expect_err("duplicate member key should reject the seed");
        assert!(error.to_string().contains("duplicate key"));

        let admin = users::Entity::find()
            .filter(users::Column::Email.eq("admin@trustledger.gh"))
            .one(&ctx.db)
            .await
            .expect("admin lookup should succeed");
        assert!(admin.is_none(), "new admin must roll back with ledger data");

        let group = groups::Entity::find_by_id(group_id)
            .one(&ctx.db)
            .await
            .expect("group lookup should succeed")
            .expect("group should remain present");
        assert_eq!(group.name, "Rollback Sentinel");

        let final_transaction_count = transactions::Entity::find()
            .filter(transactions::Column::MoolreRef.starts_with("DEMO-"))
            .count(&ctx.db)
            .await
            .expect("transaction count should succeed");
        assert_eq!(final_transaction_count, transaction_count);
    })
    .await;
}
