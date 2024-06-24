use crate::{matching::identifiers::Identifiers, prelude::*};

use futures::future::BoxFuture;
use futures_util::FutureExt;
use std::future::Future;

type Handler<'req, 'ids, 'resp, Context> = dyn Fn(&'req Request<'_>, &'ids Identifiers<'_>, &Context) -> BoxFuture<'resp, Response>
    + 'resp;

pub struct AsynchronousFunction<'req, 'ids, 'resp, Context: 'static>
where
    'req: 'resp,
    'ids: 'resp,
{
    callback: Box<Handler<'req, 'ids, 'resp, Context>>,
}

impl<'args, 'req, 'ids, 'resp, Context: 'static> AsynchronousFunction<'req, 'ids, 'resp, Context>
where
    'args: 'req,
    'args: 'ids,
    'req: 'resp,
    'ids: 'resp,
{
    pub async fn invoke(
        &self,
        req: &'args Request<'_>,
        ids: &'args Identifiers<'_>,
        server: &Context,
    ) -> Response {
        return (self.callback)(req, ids, server).await;
    }
}

impl<'req, 'ids, 'resp, Context: 'static, Fut, F> From<F>
    for AsynchronousFunction<'req, 'ids, 'resp, Context>
where
    Fut: Future<Output = Response> + Send + 'resp,
    F: Fn(&'req Request<'_>, &'ids Identifiers<'_>, &Context) -> Fut + 'resp,
{
    fn from(value: F) -> Self {
        return AsynchronousFunction {
            callback: Box::new(move |req, ids, server| {
                return value(req, ids, server).boxed();
            }),
        };
    }
}

// impl<'a, 'b, Fut, F> From<F> for AsyncFunction<'a, 'b>
// where
//     'a: 'b,
//     Fut: Future<Output = ()> + Send + 'b,
//     F: Fn(&'a str) -> Fut + 'b,
// {
//     fn from(item: F) -> Self {
//         return AsyncFunction {
//             callback: Box::new(move |a| Box::pin(item(a))),
//         };
//     }
// }
