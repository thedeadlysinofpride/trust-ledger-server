use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum SavingsGoals {
    Table,
    Id,
    MemberId,
    Name,
    TargetAmount,
    CurrentAmount,
    StartDate,
    EndDate,
    Frequency,
    Mode,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SavingsGoals::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SavingsGoals::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SavingsGoals::MemberId).integer().not_null())
                    .col(ColumnDef::new(SavingsGoals::Name).string().not_null())
                    .col(ColumnDef::new(SavingsGoals::TargetAmount).decimal().not_null())
                    .col(ColumnDef::new(SavingsGoals::CurrentAmount).decimal().default(0))
                    .col(ColumnDef::new(SavingsGoals::StartDate).date_time().not_null())
                    .col(ColumnDef::new(SavingsGoals::EndDate).date_time().not_null())
                    .col(ColumnDef::new(SavingsGoals::Frequency).string().not_null())
                    .col(ColumnDef::new(SavingsGoals::Mode).string().not_null())
                    .col(ColumnDef::new(SavingsGoals::Status).string().default("active"))
                    .col(ColumnDef::new(SavingsGoals::CreatedAt).date_time().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(SavingsGoals::UpdatedAt).date_time().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_savings_member")
                            .from(SavingsGoals::Table, SavingsGoals::MemberId)
                            .to(crate::m20260608_024127_members_table::Members::Table, crate::m20260608_024127_members_table::Members::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SavingsGoals::Table).to_owned())
            .await
    }
}
