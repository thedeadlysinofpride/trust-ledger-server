use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum TrustScoreHistory {
    Table,
    Id,
    MemberId,
    Score,
    Change,
    Reason,
    Date,
    CreatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TrustScoreHistory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TrustScoreHistory::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TrustScoreHistory::MemberId).integer().not_null())
                    .col(ColumnDef::new(TrustScoreHistory::Score).integer().not_null())
                    .col(ColumnDef::new(TrustScoreHistory::Change).integer().default(0))
                    .col(ColumnDef::new(TrustScoreHistory::Reason).string().not_null())
                    .col(ColumnDef::new(TrustScoreHistory::Date).date_time().not_null())
                    .col(ColumnDef::new(TrustScoreHistory::CreatedAt).date_time().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_trust_score_member")
                            .from(TrustScoreHistory::Table, TrustScoreHistory::MemberId)
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
            .drop_table(Table::drop().table(TrustScoreHistory::Table).to_owned())
            .await
    }
}
