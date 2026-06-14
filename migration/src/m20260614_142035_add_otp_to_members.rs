use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Members {
    Table,
    OtpCode,
    OtpExpiresAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                .table(Members::Table)
                    .add_column(ColumnDef::new(Members::OtpCode).string())
                    .add_column(ColumnDef::new(Members::OtpExpiresAt).date_time())
                    .to_owned(),
            )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                .table(Members::Table)
                    .drop_column(Members::OtpCode)
                    .drop_column(Members::OtpExpiresAt)
                    .to_owned(),
            )
                .await?;

        Ok(())
    }
}
