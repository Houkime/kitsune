use crate::{consts::VERSION, error::Result, service::user::UserService, state::Zustand};
use axum::{debug_handler, extract::State, routing, Json, Router};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    schema::{posts, users},
    PgPool,
};
use kitsune_type::nodeinfo::two_one::{
    Protocol, Services, Software, TwoOne, Usage, UsageUsers, Version,
};
use simd_json::{Builder, OwnedValue};

#[debug_handler(state = crate::state::Zustand)]
#[utoipa::path(
    get,
    path = "/nodeinfo/2.1",
    responses(
        (status = 200, description = "Get response following the Nodeinfo 2.1 schema", body = TwoOne)
    ),
)]
async fn get(
    State(db_conn): State<PgPool>,
    State(user_service): State<UserService>,
) -> Result<Json<TwoOne>> {
    let mut db_conn = db_conn.get().await?;

    let total = users::table.count().get_result::<i64>(&mut db_conn).await?;
    let local_posts = posts::table
        .filter(posts::is_local.eq(true))
        .count()
        .get_result::<i64>(&mut db_conn)
        .await?;

    Ok(Json(TwoOne {
        version: Version::TwoOne,
        software: Software {
            name: env!("CARGO_PKG_NAME").into(),
            version: VERSION.into(),
            repository: env!("CARGO_PKG_REPOSITORY").into(),
            homepage: Some(env!("CARGO_PKG_HOMEPAGE").into()),
        },
        protocols: vec![Protocol::ActivityPub],
        services: Services {
            inbound: vec![],
            outbound: vec![],
        },
        open_registrations: user_service.registrations_open(),
        usage: Usage {
            users: UsageUsers {
                total: total as u64,
                active_halfyear: None,
                active_month: None,
            },
            local_comments: None,
            local_posts: local_posts as u64,
        },
        metadata: OwnedValue::null(),
    }))
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::get(get))
}
