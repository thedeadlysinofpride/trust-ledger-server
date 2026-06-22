use sea_orm_migration::prelude::*;

use crate::m20260608_030134_transactions_table::Transactions;

const INDEX_NAME: &str = "idx_transactions_dashboard_filter";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name(INDEX_NAME)
                    .table(Transactions::Table)
                    // Equality predicates precede the date range predicate used by the dashboard.
                    .col(Transactions::Status)
                    .col(Transactions::Type)
                    .col(Transactions::Date)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name(INDEX_NAME)
                    .table(Transactions::Table)
                    .to_owned(),
            )
            .await
    }
}
