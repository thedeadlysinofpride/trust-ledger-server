use sea_orm_migration::prelude::*;

use crate::{m20260608_022801_groups_table::Groups, m20260608_024127_members_table::Members};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Transactions {
    Table,
    Id,
    MemberId,
    MemberName,
    GroupId,
    GroupName,
    Amount,
    Type,
    Status,
    MoolreRef,
    Date,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Transactions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Transactions::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Transactions::MemberId).integer().not_null())
                    .col(ColumnDef::new(Transactions::MemberName).string().not_null())
                    .col(ColumnDef::new(Transactions::GroupId).integer().not_null())
                    .col(ColumnDef::new(Transactions::GroupName).string().not_null())
                    .col(ColumnDef::new(Transactions::Amount).decimal().not_null())
                    .col(ColumnDef::new(Transactions::Type).string().not_null())
                    .col(
                        ColumnDef::new(Transactions::Status)
                            .string()
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(Transactions::MoolreRef).string())
                    .col(ColumnDef::new(Transactions::Date).date_time())
                    // Foreign key to members table
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_transactions_member")
                            .from(Transactions::Table, Transactions::MemberId)
                            .to(Members::Table, Members::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    // Foreign key to groups table
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_transactions_group")
                            .from(Transactions::Table, Transactions::GroupId)
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
            .drop_table(Table::drop().table(Transactions::Table).to_owned())
            .await
    }
}
