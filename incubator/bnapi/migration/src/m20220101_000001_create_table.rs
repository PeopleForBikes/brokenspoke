use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the city table.
        manager
            .create_table(
                Table::create()
                    .table(City::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(City::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(City::Name).string().not_null())
                    .col(ColumnDef::new(City::Country).string().not_null())
                    .col(ColumnDef::new(City::State).string().not_null())
                    .col(ColumnDef::new(City::Uuid).uuid().not_null())
                    .col(ColumnDef::new(City::Population).unsigned().not_null())
                    .col(ColumnDef::new(City::Ratings).double().not_null())
                    .col(
                        ColumnDef::new(City::CreatedAt)
                            .timestamp_with_time_zone_len(0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(City::UpdatedAt)
                            .timestamp_with_time_zone_len(0)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create the BNA table.
        manager
            .create_table(
                Table::create()
                    .table(Bna::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Bna::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Bna::Neighborhoods).double().not_null())
                    .col(ColumnDef::new(Bna::Opportunity).double().not_null())
                    .col(ColumnDef::new(Bna::EssentialServices).double())
                    .col(ColumnDef::new(Bna::Retail).double().not_null())
                    .col(ColumnDef::new(Bna::Recreation).double())
                    .col(ColumnDef::new(Bna::Transit).double().not_null())
                    .col(ColumnDef::new(Bna::OverallScore).double().not_null())
                    .col(ColumnDef::new(Bna::CityId).uuid())
                    .col(
                        ColumnDef::new(Bna::CreatedAt)
                            .timestamp_with_time_zone_len(0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Bna::UpdatedAt)
                            .timestamp_with_time_zone_len(0)
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Bna::Table, Bna::CityId)
                            .to(City::Table, City::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create the community_survey table.
        manager
            .create_table(
                Table::create()
                    .table(CommunitySurvey::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CommunitySurvey::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(CommunitySurvey::Network).double().not_null())
                    .col(
                        ColumnDef::new(CommunitySurvey::Awareness)
                            .double()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CommunitySurvey::Safety).double().not_null())
                    .col(
                        ColumnDef::new(CommunitySurvey::Ridership)
                            .double()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CommunitySurvey::Total).double().not_null())
                    .col(
                        ColumnDef::new(CommunitySurvey::Responses)
                            .unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CommunitySurvey::CityId).uuid())
                    .col(
                        ColumnDef::new(CommunitySurvey::CreatedAt)
                            .timestamp_with_time_zone_len(0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CommunitySurvey::UpdatedAt)
                            .timestamp_with_time_zone_len(0)
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(CommunitySurvey::Table, CommunitySurvey::CityId)
                            .to(City::Table, City::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create the infrastructure table.
        manager
            .create_table(
                Table::create()
                    .table(Infrastructure::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Infrastructure::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Infrastructure::LowStressMiles).double())
                    .col(ColumnDef::new(Infrastructure::HighStressMiles).double())
                    .col(ColumnDef::new(Infrastructure::CityId).uuid())
                    .col(
                        ColumnDef::new(Infrastructure::CreatedAt)
                            .timestamp_with_time_zone_len(0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Infrastructure::UpdatedAt)
                            .timestamp_with_time_zone_len(0)
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Infrastructure::Table, Infrastructure::CityId)
                            .to(City::Table, City::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Bna::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(City::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(CommunitySurvey::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Infrastructure::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum City {
    Table,
    Id,
    Name,
    Country,
    State,
    Uuid,
    Population,
    Ratings,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Bna {
    Table,
    Id,
    Neighborhoods,
    Opportunity,
    EssentialServices,
    Retail,
    Recreation,
    Transit,
    OverallScore,
    CityId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum CommunitySurvey {
    Table,
    Id,
    Network,
    Awareness,
    Safety,
    Ridership,
    Total,
    Responses,
    CityId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Infrastructure {
    Table,
    Id,
    LowStressMiles,
    HighStressMiles,
    CityId,
    CreatedAt,
    UpdatedAt,
}
