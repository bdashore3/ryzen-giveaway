use serenity::prelude::TypeMapKey;
use sqlx::PgPool;

pub struct ConnectionPool;

impl TypeMapKey for ConnectionPool {
    type Value = PgPool;
}
