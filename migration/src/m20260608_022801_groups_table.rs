use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Groups {
    Table,
    Id,
    Name,
    Code,
    Description,
    MemberCount,
    TotalContributed,
    ContributionAmount,
    Frequency,
    NextPayoutDate,
    IsActive,
    CreatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Groups::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Groups::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Groups::Name).string().not_null())
                    .col(
                        ColumnDef::new(Groups::Code)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Groups::Description).text())
                    .col(ColumnDef::new(Groups::MemberCount).integer().default(0))
                    .col(
                        ColumnDef::new(Groups::TotalContributed)
                            .decimal()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Groups::ContributionAmount)
                            .decimal()
                            .default(0),
                    )
                    .col(ColumnDef::new(Groups::Frequency).string().not_null())
                    .col(
                        ColumnDef::new(Groups::NextPayoutDate)
                            .date_time()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Groups::IsActive).boolean().default(true))
                    .col(
                        ColumnDef::new(Groups::CreatedAt)
                            .date_time()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Groups::Table).to_owned())
            .await
    }
}
