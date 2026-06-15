#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;

mod m20260608_022801_groups_table;
mod m20260608_024127_members_table;
mod m20260608_025454_contributions_table;
mod m20260608_030134_transactions_table;
mod m20260614_142035_add_otp_to_members;
mod m20260614_154403_make_group_id_optional;
mod m20260615_092957_savings_goals;
mod m20260615_093113_savings_deposits;
mod m20260615_094954_trust_score_history;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20260608_022801_groups_table::Migration),
            Box::new(m20260608_024127_members_table::Migration),
            Box::new(m20260608_025454_contributions_table::Migration),
            Box::new(m20260608_030134_transactions_table::Migration),
            Box::new(m20260614_142035_add_otp_to_members::Migration),
            Box::new(m20260614_154403_make_group_id_optional::Migration),
            Box::new(m20260615_092957_savings_goals::Migration),
            Box::new(m20260615_093113_savings_deposits::Migration),
            Box::new(m20260615_094954_trust_score_history::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}