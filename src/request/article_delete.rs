use crate::{session, article, request};
use indexmap::IndexMap;
use futures::prelude::*;
use seed::fetch;

type ServerData = IndexMap<(), ()>;

pub fn delete_article<Ms: 'static>(
    session: &session::Session,
    slug: &article::slug::Slug,
    f: fn(Result<(), Vec<String>>) -> Ms,
) -> impl Future<Item=Ms, Error=Ms>  {
    request::new_api_request(
        &format!("articles/{}", slug.as_str()),
        session.viewer().map(|viewer| &viewer.credentials)
    )
        .method(fetch::Method::Delete)
        .fetch_json_data(move |data_result: fetch::ResponseDataResult<ServerData>| {
            f(data_result
                .map(move |_| ())
                .map_err(request::fail_reason_into_errors)
            )
        })
}