use crate::application::contracts::{KeysetDirection, KeysetPageCursor, PaginationMode};
use crate::web::{error::AppError, handlers::dto::SearchQueryParams};
use serde::{Deserialize, Serialize};
use sqlx::{Execute, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

#[derive(Clone, Copy, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    #[default]
    Relevance,
    UuidAsc,
    UuidDesc,
}
pub struct Options {
    pub text: String,
    pub must: Vec<String>,
    pub should: Vec<String>,
    pub excluded: Vec<String>,
    pub ids: Vec<(char, Uuid)>,
    pub sort: Sort,
}
impl Options {
    pub fn parse(query: &SearchQueryParams) -> Result<Self, AppError> {
        let tags = query.tag_query.clone().unwrap_or_default();
        let mut result = Self {
            text: String::new(),
            must: vec![],
            should: tags.should,
            excluded: tags.must_not,
            ids: vec![],
            sort: query.sort.unwrap_or_default(),
        };
        let mut accept = |token: &str| -> Result<bool, AppError> {
            if let Some(raw) = token.strip_prefix("id:") {
                let op = raw.chars().next().unwrap_or(' ');
                if !['<', '>', '='].contains(&op) {
                    return Err(AppError::bad_request("Use id:<UUID, id:>UUID or id:=UUID"));
                }
                let id = raw[op.len_utf8()..].trim_matches(|c| c == '{' || c == '}');
                result.ids.push((
                    op,
                    Uuid::parse_str(id)
                        .map_err(|_| AppError::bad_request("Invalid UUID in id filter"))?,
                ));
                return Ok(true);
            }
            Ok(false)
        };
        for token in tags.must {
            if !accept(&token)? {
                result.must.push(token);
            }
        }
        let mut text = vec![];
        for token in query.text_query.as_deref().unwrap_or("").split_whitespace() {
            if !accept(token)? {
                text.push(token);
            }
        }
        result.text = text.join(" ");
        Ok(result)
    }
    pub fn advanced(&self) -> bool {
        self.sort != Sort::Relevance || !self.text.is_empty() || !self.ids.is_empty()
    }
}
#[derive(Clone)]
pub struct PostgresSearchRepository {
    pool: PgPool,
}
pub struct Page {
    pub rows: Vec<(Uuid, i64)>,
    pub has_next: bool,
    pub has_prev: bool,
    pub next: Option<KeysetPageCursor>,
    pub prev: Option<KeysetPageCursor>,
    pub total_pages: Option<i64>,
}
#[derive(Serialize)]
pub struct Response<T> {
    #[serde(flatten)]
    pub data: T,
    pub has_next: bool,
    pub has_prev: bool,
    pub next_cursor: Option<KeysetPageCursor>,
    pub prev_cursor: Option<KeysetPageCursor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_pages: Option<i64>,
}
impl Page {
    pub fn response<T>(self, data: T) -> Response<T> {
        Response {
            data,
            has_next: self.has_next,
            has_prev: self.has_prev,
            next_cursor: self.next,
            prev_cursor: self.prev,
            total_pages: self.total_pages,
        }
    }
}
fn base<'a>(options: &'a Options, user: Option<Uuid>) -> QueryBuilder<'a, Postgres> {
    let (table, links, foreign) = if user.is_some() {
        ("collections", "collection_tags", "collection_id")
    } else {
        ("posts", "post_tags", "post_id")
    };
    let mut sql = QueryBuilder::new(format!(
        "SELECT e.id, (SELECT count(*) FROM {links} st JOIN tags t ON t.id=st.tag_id WHERE st.{foreign}=e.id AND t.name=ANY("
    ));
    sql.push_bind(&options.should)
        .push(format!(")) AS score FROM {table} e WHERE true"));
    if let Some(user) = user {
        sql.push(" AND e.owner_id=").push_bind(user);
    }
    if !options.text.is_empty() {
        sql.push(" AND (e.title ILIKE ")
            .push_bind(format!("%{}%", options.text))
            .push(" OR COALESCE(e.description,'') ILIKE ")
            .push_bind(format!("%{}%", options.text))
            .push(")");
    }
    for (op, id) in &options.ids {
        sql.push(" AND e.id ")
            .push(op.to_string())
            .push(" ")
            .push_bind(*id);
    }
    sql.push(" AND NOT EXISTS (SELECT 1 FROM unnest(").push_bind(&options.must).push(format!("::text[]) wanted(name) WHERE NOT EXISTS (SELECT 1 FROM {links} lt JOIN tags t ON t.id=lt.tag_id WHERE lt.{foreign}=e.id AND t.name=wanted.name))"));
    sql.push(format!(" AND NOT EXISTS (SELECT 1 FROM {links} lt JOIN tags t ON t.id=lt.tag_id WHERE lt.{foreign}=e.id AND t.name=ANY(")).push_bind(&options.excluded).push("))");
    sql
}
impl PostgresSearchRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn summaries(
        &self,
        user: Uuid,
        ids: &[Uuid],
    ) -> Result<Vec<crate::domain::model::CollectionSummary>, AppError> {
        use crate::storage::postgres::dto::TagResponse;
        use sqlx::types::Json;
        let rows: Vec<(Uuid, String, Option<String>, Option<Uuid>, i64, Json<Vec<TagResponse>>)> = sqlx::query_as(
            "SELECT c.id,c.title,c.description,c.cover_file_id, (SELECT count(*) FROM collection_items WHERE collection_id=c.id),
            COALESCE((SELECT jsonb_agg(jsonb_build_object('id',t.id,'name',t.name,'category',t.category,'count',t.post_count)) FROM collection_tags ct JOIN tags t ON t.id=ct.tag_id WHERE ct.collection_id=c.id),'[]'::jsonb)
            FROM collections c WHERE c.owner_id=$1 AND c.id=ANY($2)")
            .bind(user).bind(ids).fetch_all(&self.pool).await.map_err(db_error)?;
        let mut by_id: std::collections::HashMap<_, _> = rows
            .into_iter()
            .map(|(id, title, description, cover, item_count, tags)| {
                (
                    id,
                    crate::domain::model::CollectionSummary {
                        id,
                        title,
                        description: description.unwrap_or_default(),
                        cover,
                        item_count,
                        tags: tags.0.into_iter().map(Into::into).collect(),
                    },
                )
            })
            .collect();
        Ok(ids.iter().filter_map(|id| by_id.remove(id)).collect())
    }
    pub async fn search(
        &self,
        options: &Options,
        query: &SearchQueryParams,
        user: Option<Uuid>,
    ) -> Result<Page, AppError> {
        let cursor = query.cursor.clone().unwrap_or_default();
        let offset = matches!(cursor.mode, Some(PaginationMode::Offset));
        let limit = cursor.limit.unwrap_or(20).clamp(1, 100);
        let reverse = !offset && matches!(cursor.direction, Some(KeysetDirection::Prev));
        let use_cursor = !offset && cursor.last_id.is_some();
        let asc = (options.sort == Sort::UuidAsc) != reverse;
        let cmp = if asc { ">" } else { "<" };
        let order = if asc { "ASC" } else { "DESC" };
        let mut total_pages = None;
        if offset {
            let mut count = base(options, user);
            let inner = count.sql().to_string();
            // Wrap the existing builder without changing its bound arguments.
            let sql = format!("SELECT count(*) FROM ({inner}) matched");
            let args = count
                .build()
                .take_arguments()
                .map_err(|e| AppError::internal(e.to_string()))?
                .unwrap();
            let total: i64 = sqlx::query_scalar_with(&sql, args)
                .fetch_one(&self.pool)
                .await
                .map_err(db_error)?;
            total_pages = Some((total + limit - 1) / limit);
        }
        let mut inner = base(options, user);
        let inner_sql = inner.sql().to_string();
        let args = inner
            .build()
            .take_arguments()
            .map_err(|e| AppError::internal(e.to_string()))?
            .unwrap();
        let mut sql = format!("WITH matched AS ({inner_sql}) SELECT id,score FROM matched");
        // Continue using bind indices after the base query.
        let mut args = args;
        use sqlx::Arguments;
        let mut n = args.len();
        if use_cursor {
            if options.sort == Sort::Relevance {
                n += 1;
                args.add(cursor.last_score.unwrap_or(0.0))
                    .map_err(|e| AppError::internal(e.to_string()))?;
                sql.push_str(&format!(" WHERE (score::float8,id) {cmp} (${n},"));
                n += 1;
                args.add(cursor.last_id.unwrap())
                    .map_err(|e| AppError::internal(e.to_string()))?;
                sql.push_str(&format!("${n})"));
            } else {
                n += 1;
                args.add(cursor.last_id.unwrap())
                    .map_err(|e| AppError::internal(e.to_string()))?;
                sql.push_str(&format!(" WHERE id {cmp} ${n}"));
            }
        }
        sql.push_str(" ORDER BY ");
        if options.sort == Sort::Relevance {
            sql.push_str(&format!("score {order}, "));
        }
        sql.push_str(&format!("id {order}"));
        n += 1;
        args.add(limit + 1)
            .map_err(|e| AppError::internal(e.to_string()))?;
        sql.push_str(&format!(" LIMIT ${n}"));
        if offset {
            n += 1;
            args.add(cursor.page.unwrap_or(0).max(0).saturating_mul(limit))
                .map_err(|e| AppError::internal(e.to_string()))?;
            sql.push_str(&format!(" OFFSET ${n}"));
        }
        let mut rows: Vec<(Uuid, i64)> = sqlx::query_as_with(&sql, args)
            .fetch_all(&self.pool)
            .await
            .map_err(db_error)?;
        let more = rows.len() > limit as usize;
        rows.truncate(limit as usize);
        if reverse {
            rows.reverse();
        }
        let has_next = if reverse { use_cursor } else { more };
        let has_prev = if reverse { more } else { use_cursor };
        let make = |(id, score): (Uuid, i64), direction| KeysetPageCursor {
            mode: PaginationMode::Keyset,
            direction,
            last_id: id,
            last_score: score as f64,
            limit,
        };
        let next = if has_next {
            rows.last().copied().map(|r| make(r, KeysetDirection::Next))
        } else {
            None
        };
        let prev = if has_prev {
            rows.first()
                .copied()
                .map(|r| make(r, KeysetDirection::Prev))
        } else {
            None
        };
        Ok(Page {
            rows,
            has_next,
            has_prev,
            next,
            prev,
            total_pages,
        })
    }
}
fn db_error(err: sqlx::Error) -> AppError {
    log::error!("advanced search: {err}");
    AppError::internal("Could not search")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn query(value: serde_json::Value) -> SearchQueryParams {
        serde_json::from_value(value).unwrap()
    }
    #[test]
    fn id_ranges_preserve_other_filters_and_text() {
        let id = "019d5f0c-9974-79a1-ba95-1aaa28dd91c0";
        let options=Options::parse(&query(serde_json::json!({"sort":"uuid_asc","tag_query":{"must":["landscape",format!("id:>{{{id}}}")],"should":["sun"],"must_not":["rain"]},"text_query":format!("blue sky id:<{id}")}))).unwrap();
        assert_eq!(options.ids.len(), 2);
        assert_eq!(options.ids[0].0, '>');
        assert_eq!(options.ids[1].0, '<');
        assert_eq!(options.text, "blue sky");
        assert_eq!(options.must, vec!["landscape"]);
        assert!(options.advanced());
        let sql = base(&options, None).sql().to_string();
        assert!(sql.contains("FROM posts e"));
        assert!(sql.contains("e.id >"));
        assert!(sql.contains("e.id <"));
        assert!(!sql.contains(id));
        assert!(!sql.contains("landscape"));
        let sql = base(&options, Some(Uuid::nil())).sql().to_string();
        assert!(sql.contains("e.owner_id="));
        assert!(sql.contains("FROM collections e"));
    }
    #[test]
    fn bad_id_filters_and_unknown_sort_are_rejected() {
        for token in [
            "id:bad",
            "id:>bad",
            "id:>=019d5f0c-9974-79a1-ba95-1aaa28dd91c0",
        ] {
            assert!(Options::parse(&query(serde_json::json!({"text_query":token}))).is_err());
        }
        assert!(
            serde_json::from_value::<SearchQueryParams>(serde_json::json!({"sort":"oops"}))
                .is_err()
        );
        assert!(
            !Options::parse(&query(serde_json::json!({})))
                .unwrap()
                .advanced()
        );
    }
}
