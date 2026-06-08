use sea_orm_migration::prelude::*;

use crate::m20260608_022801_groups_table::Groups;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Members {
    Table,
    Id,
    Name,
    Phone,
    TrustScore,
    GroupId,
    TotalContributed,
    OnTimePayments,
    TotalPayments,
    LastMissedPayment,
    Status,
    JoinedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Members::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Members::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Members::Name).string().not_null())
                    .col(ColumnDef::new(Members::Phone).string().not_null())
                    .col(ColumnDef::new(Members::TrustScore).integer().default(100))
                    .col(ColumnDef::new(Members::GroupId).integer().not_null())
                    .col(
                        ColumnDef::new(Members::TotalContributed)
                            .decimal()
                            .default(0),
                    )
                    .col(ColumnDef::new(Members::OnTimePayments).integer().default(0))
                    .col(ColumnDef::new(Members::TotalPayments).integer().default(0))
                    .col(ColumnDef::new(Members::LastMissedPayment).date_time())
                    .col(ColumnDef::new(Members::Status).string().default("active"))
                    .col(ColumnDef::new(Members::JoinedAt).date_time())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_members_group")
                            .from(Members::Table, Members::GroupId)
                            .to(Groups::Table, Groups::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Members::Table).to_owned())
            .await
    }
}
