use chrono::NaiveDateTime;
use diesel::pg::Pg;
use diesel::pg::PgConnection;
use diesel::prelude::*;

use crate::dal::schema::post;
use crate::dal::schema::post::dsl::post as all_posts;
use crate::util::time::get_now;
use diesel::diesel_infix_operator;
use diesel::expression::AsExpression;
use diesel::Identifiable;
use diesel::Insertable;
use diesel::Queryable;
use serde::{Deserialize, Serialize};
use serde_json::Value;

diesel_infix_operator!(PgJsonContains, " @> ");

// Normally you would put this on a trait instead
fn json_contains<T, U>(left: T, right: U) -> PgJsonContains<T, U::Expression>
where
    T: Expression,
    U: AsExpression<T::SqlType>,
{
    PgJsonContains::new(left, right.as_expression())
}

// Because diesel still doesn't support enum, So for convenience, I do it by hand
const ABOUT: i32 = 1;
const POST: i32 = 2;
const FRIEND: i32 = 3;
pub const LIMIT: i64 = 5;

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, AsChangeset, Identifiable)]
#[table_name = "post"]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub subtitle: String,
    pub raw_content: String,
    pub rendered_content: String,
    pub create_time: NaiveDateTime,
    #[serde(default = "get_now")]
    pub modify_time: NaiveDateTime,
    pub post_type: i32,
    pub hit_time: i32,
    pub published: bool,
    pub slug_url: String,
    pub enable_comment: bool,
    pub tag: Value,
}
impl Post {
    pub fn query_all(conn: &PgConnection) -> Vec<Post> {
        all_posts.order(post::id.desc()).load::<Post>(conn).unwrap()
    }
    fn published() -> post::BoxedQuery<'static, Pg> {
        all_posts
            .filter(post::published.eq(true))
            .order(post::create_time.desc())
            .into_boxed()
    }
    pub fn query_all_published_post(conn: &PgConnection) -> Vec<Post> {
        Post::published()
            .filter(post::post_type.eq(POST))
            .load::<Post>(conn)
            .expect("Error loading posts")
    }

    pub fn query_latest_about(conn: &PgConnection) -> Vec<Post> {
        Post::published()
            .filter(post::post_type.eq(ABOUT))
            .load::<Post>(conn)
            .expect("Error loading about posts")
    }

    pub fn query_latest_friend(conn: &PgConnection) -> Vec<Post> {
        Post::published()
            .filter(post::post_type.eq(FRIEND))
            .load::<Post>(conn)
            .expect("Error loading friend posts")
    }

    pub fn query_latest_five_post(conn: &PgConnection) -> (Vec<Post>, bool) {
        let mut posts = Post::published()
            .filter(post::post_type.eq(POST))
            .limit(6)
            .load::<Post>(conn)
            .expect("Error loading posts");
        let mut more = false;
        if posts.len() > 5 {
            more = true;
            posts.pop();
        }
        (posts, more)
    }
    pub fn pagination_query(offset: i64, conn: &PgConnection) -> Vec<Post> {
        Post::published()
            .filter(post::post_type.eq(POST))
            .offset(offset * LIMIT)
            .limit(LIMIT)
            .load::<Post>(conn)
            .expect("Error loading posts")
    }

    pub fn query_by_tag(query_tag: Value, conn: &PgConnection) -> Vec<Post> {
        Post::published()
            .filter(json_contains(post::tag, query_tag))
            .load::<Post>(conn)
            .expect("Error loading about posts")
    }

    pub fn query_by_id(conn: &PgConnection, id: i32) -> Vec<Post> {
        all_posts
            .find(id)
            .load::<Post>(conn)
            .expect("Error finding post")
    }
    pub fn query_by_slug_url(conn: &PgConnection, slug_url: &str) -> Vec<Post> {
        all_posts
            .filter(post::slug_url.eq(slug_url))
            .load::<Post>(conn)
            .expect("Error when finding user by email")
    }
    pub fn query_ten_hottest_posts(conn: &PgConnection) -> Vec<Post> {
        all_posts
            .filter(post::post_type.eq(POST))
            .order(post::hit_time.desc())
            .limit(10)
            .load::<Post>(conn)
            .expect("Error when loading posts")
    }
    pub fn delete_with_id(conn: &PgConnection, id: i32) -> bool {
        diesel::delete(all_posts.find(id)).execute(conn).is_ok()
    }
    pub fn update(conn: &PgConnection, post: &Post) -> bool {
        diesel::update(post).set(post).execute(conn).is_ok()
    }
    pub fn increase_hit_time(conn: &PgConnection, id: i32, hit_time: i32) -> bool {
        diesel::update(all_posts.find(id))
            .set(post::hit_time.eq(hit_time))
            .execute(conn)
            .is_ok()
    }
}
#[derive(Insertable, Deserialize, Serialize)]
#[table_name = "post"]
pub struct NewPost {
    pub title: String,
    pub subtitle: String,
    pub raw_content: String,
    pub rendered_content: String,
    #[serde(default = "get_now")]
    pub create_time: NaiveDateTime,
    #[serde(default = "get_now")]
    pub modify_time: NaiveDateTime,
    pub post_type: i32,
    #[serde(default)]
    pub hit_time: i32,
    pub published: bool,
    pub slug_url: String,
    pub enable_comment: bool,
    pub tag: Value,
}
impl NewPost {
    pub fn insert(new_post: &NewPost, conn: &PgConnection) -> bool {
        diesel::insert_into(post::table)
            .values(new_post)
            .execute(conn)
            .is_ok()
    }
}

#[derive(Deserialize, Serialize)]
pub struct PostView {
    pub id: i32,
    pub title: String,
    pub subtitle: String,
    pub create_time: NaiveDateTime,
    pub post_type: i32,
    pub hit_time: i32,
    pub published: bool,
    pub slug_url: String,
    pub enable_comment: bool,
    pub tag: Value,
}
impl PostView {
    pub fn model_convert_to_postview(post: &Post) -> PostView {
        PostView {
            id: post.id,
            title: post.title.to_string(),
            subtitle: post.subtitle.to_string(),
            create_time: post.create_time,
            post_type: post.post_type,
            published: post.published,
            hit_time: post.hit_time,
            slug_url: post.slug_url.to_string(),
            enable_comment: post.enable_comment,
            tag: post.tag.to_owned(),
        }
    }
}
// and in the structure
