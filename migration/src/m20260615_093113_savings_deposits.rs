use sea_orm_migration::prelude::*;

use crate::m20260615_092957_savings_goals::SavingsGoals;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum SavingsDeposits {
    Table,
    Id,
    GoalId,
    Amount,
    Date,
    MoolreRef,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SavingsDeposits::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SavingsDeposits::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SavingsDeposits::GoalId).integer().not_null())
                    .col(ColumnDef::new(SavingsDeposits::Amount).decimal().not_null())
                    .col(ColumnDef::new(SavingsDeposits::Date).date_time().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(SavingsDeposits::MoolreRef).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_deposit_goal")
                            .from(SavingsDeposits::Table, SavingsDeposits::GoalId)
                            .to(SavingsGoals::Table, SavingsGoals::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SavingsDeposits::Table).to_owned())
            .await
    }
}
