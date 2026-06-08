#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;

mod m20260608_022801_groups_table;
mod m20260608_024127_members_table;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20260608_022801_groups_table::Migration),
            Box::new(m20260608_024127_members_table::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}