use sea_orm_migration::prelude::*;

use crate::{m20260608_022801_groups_table::Groups, m20260608_024127_members_table::Members};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Contributions {
    Table,
    Id,
    MemberId,
    GroupId,
    Amount,
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
                    .table(Contributions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Contributions::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Contributions::MemberId).integer().not_null())
                    .col(ColumnDef::new(Contributions::GroupId).integer().not_null())
                    .col(ColumnDef::new(Contributions::Amount).decimal().not_null())
                    .col(
                        ColumnDef::new(Contributions::Status)
                            .string()
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(Contributions::MoolreRef).string())
                    .col(ColumnDef::new(Contributions::Date).date_time())
                    // Foreign key to members table
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_contributions_member")
                            .from(Contributions::Table, Contributions::MemberId)
                            .to(Members::Table, Members::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    // Foreign key to groups table
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_contributions_group")
                            .from(Contributions::Table, Contributions::GroupId)
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
            .drop_table(Table::drop().table(Contributions::Table).to_owned())
            .await
    }
}
