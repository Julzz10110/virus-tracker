use crate::models::{GrowthMetric, Mutation, VariantQueryParams, VariantStats, VirusVariant};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    FromRow, QueryBuilder, SqlitePool,
};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct Storage {
    pool: Arc<SqlitePool>,
}

impl Storage {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let options = database_url
            .parse::<SqliteConnectOptions>()?
            .create_if_missing(true)
            .foreign_keys(true);

        let max_connections = if database_url.contains(":memory:") {
            1
        } else {
            5
        };

        let pool = SqlitePoolOptions::new()
            .max_connections(max_connections)
            .connect_with(options)
            .await?;
        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        sqlx::migrate!("./migrations")
            .run(&*self.pool)
            .await
            .map_err(sqlx::Error::from)
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn insert_variant_with_mutations(
        &self,
        variant: &VirusVariant,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO variants (id, name, lineage, location, date, sequence, created_at)
            VALUES (?, ?, ?, ?, ?, ?, datetime('now'))
            "#,
        )
        .bind(&variant.id)
        .bind(&variant.name)
        .bind(&variant.lineage)
        .bind(&variant.location)
        .bind(&variant.date)
        .bind(&variant.sequence)
        .execute(&mut *tx)
        .await?;

        if !variant.mutations.is_empty() {
            let mut builder: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
                "INSERT INTO variant_mutations (variant_id, gene, position, reference, mutation) ",
            );
            builder.push_values(&variant.mutations, |mut b, mutation| {
                b.push_bind(&variant.id)
                    .push_bind(&mutation.gene)
                    .push_bind(mutation.position as i64)
                    .push_bind(mutation.reference.to_string())
                    .push_bind(mutation.mutation.to_string());
            });

            builder.build().execute(&mut *tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn list_variants(
        &self,
        params: &VariantQueryParams,
    ) -> Result<Vec<VirusVariant>, sqlx::Error> {
        let mut sql = String::from(
            "SELECT id, name, lineage, location, date, sequence FROM variants WHERE 1 = 1",
        );
        if params.lineage.is_some() {
            sql.push_str(" AND LOWER(lineage) = LOWER(?)");
        }
        if params.location.is_some() {
            sql.push_str(" AND LOWER(location) = LOWER(?)");
        }

        let mut query = sqlx::query_as::<_, VariantRow>(&sql);
        if let Some(lineage) = &params.lineage {
            query = query.bind(lineage);
        }
        if let Some(location) = &params.location {
            query = query.bind(location);
        }

        #[derive(FromRow)]
        struct VariantRow {
            id: String,
            name: String,
            lineage: String,
            location: String,
            date: String,
            sequence: String,
        }

        let records: Vec<VariantRow> = query.fetch_all(&*self.pool).await?;

        if records.is_empty() {
            return Ok(Vec::new());
        }

        #[derive(FromRow)]
        struct MutationRow {
            variant_id: String,
            gene: String,
            position: i64,
            reference: String,
            mutation: String,
        }

        let mut builder =
            QueryBuilder::new("SELECT variant_id, gene, position, reference, mutation FROM variant_mutations WHERE variant_id IN (");
        let mut separated = builder.separated(", ");
        for record in &records {
            separated.push_bind(&record.id);
        }
        builder.push(")");

        let mutation_rows: Vec<MutationRow> =
            builder.build_query_as().fetch_all(&*self.pool).await?;

        let mut mutation_map: HashMap<String, Vec<Mutation>> = HashMap::new();
        for row in mutation_rows {
            mutation_map
                .entry(row.variant_id)
                .or_default()
                .push(Mutation {
                    gene: row.gene,
                    position: row.position as u32,
                    reference: row.reference.chars().next().unwrap_or_default(),
                    mutation: row.mutation.chars().next().unwrap_or_default(),
                });
        }

        Ok(records
            .into_iter()
            .map(|row| {
                let mutations = mutation_map.remove(&row.id).unwrap_or_default();
                VirusVariant {
                    id: row.id,
                    name: row.name,
                    lineage: row.lineage,
                    location: row.location,
                    date: row.date,
                    sequence: row.sequence,
                    mutations,
                }
            })
            .collect())
    }

    pub async fn get_stats(&self) -> Result<VariantStats, sqlx::Error> {
        let totals: (i64,) = sqlx::query_as("SELECT COUNT(*) as total FROM variants")
            .fetch_one(&*self.pool)
            .await?;

        let unique_locations: (i64,) = sqlx::query_as(
            "SELECT COUNT(DISTINCT LOWER(location)) as unique_locations FROM variants",
        )
        .fetch_one(&*self.pool)
        .await?;

        let total_mutations: (i64,) =
            sqlx::query_as("SELECT COUNT(*) as total_mutations FROM variant_mutations")
                .fetch_one(&*self.pool)
                .await?;

        Ok(VariantStats {
            total_variants: totals.0 as usize,
            unique_locations: unique_locations.0 as usize,
            total_mutations: total_mutations.0 as usize,
            average_mutations_per_variant: if totals.0 > 0 {
                total_mutations.0 as f64 / totals.0 as f64
            } else {
                0.0
            },
        })
    }

    pub async fn get_reference_sequence(&self) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT sequence FROM reference_genome WHERE id = 1 LIMIT 1")
                .fetch_optional(&*self.pool)
                .await?;

        Ok(row.map(|tuple| tuple.0))
    }

    pub async fn set_reference_sequence(&self, sequence: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO reference_genome (id, sequence, updated_at)
            VALUES (1, ?, datetime('now'))
            ON CONFLICT(id) DO UPDATE SET sequence = excluded.sequence, updated_at = excluded.updated_at
            "#,
        )
        .bind(sequence)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_user(
        &self,
        user_id: &str,
        email: &str,
        password_hash: &str,
        roles: &[String],
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, created_at)
            VALUES (?, ?, ?, datetime('now'))
            "#,
        )
        .bind(user_id)
        .bind(email)
        .bind(password_hash)
        .execute(&mut *tx)
        .await?;

        for role in roles {
            sqlx::query(
                r#"
                INSERT OR IGNORE INTO roles (name) VALUES (?)
                "#,
            )
            .bind(role)
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                r#"
                INSERT OR IGNORE INTO user_roles (user_id, role_id)
                SELECT ?, id FROM roles WHERE name = ?
                "#,
            )
            .bind(user_id)
            .bind(role)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_user_with_roles(
        &self,
        email: &str,
    ) -> Result<Option<UserRecord>, sqlx::Error> {
        #[derive(FromRow)]
        struct UserRow {
            id: String,
            email: String,
            password_hash: String,
        }

        let user: Option<UserRow> = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, password_hash FROM users WHERE email = ?",
        )
        .bind(email)
        .fetch_optional(&*self.pool)
        .await?;

        let Some(user) = user else {
            return Ok(None);
        };

        let roles: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT roles.name
            FROM roles
            INNER JOIN user_roles ON user_roles.role_id = roles.id
            WHERE user_roles.user_id = ?
            "#,
        )
        .bind(&user.id)
        .fetch_all(&*self.pool)
        .await?;

        Ok(Some(UserRecord {
            id: user.id,
            email: user.email,
            password_hash: user.password_hash,
            roles,
        }))
    }

    pub async fn list_users(&self) -> Result<Vec<UserSummary>, sqlx::Error> {
        #[derive(FromRow)]
        struct UserRow {
            id: String,
            email: String,
            created_at: String,
        }

        let users: Vec<UserRow> = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, created_at FROM users ORDER BY created_at DESC",
        )
        .fetch_all(&*self.pool)
        .await?;

        let mut summaries = Vec::with_capacity(users.len());
        for user in users {
            let roles: Vec<String> = sqlx::query_scalar(
                r#"
                SELECT roles.name
                FROM roles
                INNER JOIN user_roles ON user_roles.role_id = roles.id
                WHERE user_roles.user_id = ?
                "#,
            )
            .bind(&user.id)
            .fetch_all(&*self.pool)
            .await?;

            summaries.push(UserSummary {
                id: user.id,
                email: user.email,
                created_at: user.created_at,
                roles,
            });
        }

        Ok(summaries)
    }

    pub async fn insert_growth_metric(&self, metric: &GrowthMetric) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO epidemiology_metrics
                (collected_at, total_variants, growth_rate, reproduction_number, doubling_time)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&metric.collected_at)
        .bind(metric.total_variants as i64)
        .bind(metric.growth_rate)
        .bind(metric.reproduction_number)
        .bind(metric.doubling_time)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_growth_metrics(
        &self,
        limit: usize,
    ) -> Result<Vec<GrowthMetric>, sqlx::Error> {
        #[derive(FromRow)]
        struct MetricRow {
            collected_at: String,
            total_variants: i64,
            growth_rate: Option<f64>,
            reproduction_number: Option<f64>,
            doubling_time: Option<f64>,
        }

        let rows: Vec<MetricRow> = sqlx::query_as::<_, MetricRow>(
            r#"
            SELECT collected_at, total_variants, growth_rate, reproduction_number, doubling_time
            FROM epidemiology_metrics
            ORDER BY collected_at DESC
            LIMIT ?
            "#,
        )
        .bind(limit as i64)
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| GrowthMetric {
                collected_at: row.collected_at,
                total_variants: row.total_variants as usize,
                growth_rate: row.growth_rate,
                reproduction_number: row.reproduction_number,
                doubling_time: row.doubling_time,
            })
            .collect())
    }
}

#[derive(Debug)]
pub struct UserRecord {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub roles: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct UserSummary {
    pub id: String,
    pub email: String,
    pub created_at: String,
    pub roles: Vec<String>,
}
