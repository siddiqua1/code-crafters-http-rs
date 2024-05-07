// use anyhow::Result;
// use core::future::Future;
// use core::pin::Pin;
// use std::collections::HashMap;
// use std::hash::Hash;

// use crate::core::request::Request;
// pub struct Identifiers<'a> {
//     pub path_values: HashMap<&'static str, &'a str>,
// }

// pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

// pub type SyncRouteHandler<Context> =
//     fn(req: &Request, path_vals: &Identifiers, ctx: &Context) -> Result<Vec<u8>>;

// pub type AsyncRouteHandler<Context> =
//     fn(req: &Request, path_vals: &Identifiers, ctx: &Context) -> BoxFuture<Result<Vec<u8>>>;

// trait AsyncFn<R, I, C>: Fn(R, I, C) -> <Self as AsyncFn<R, I, C>>::Fut {
//     type Fut: Future<Output = <Self as AsyncFn<R, I, C>>::Output>;
//     type Output;
// }
// impl<R, I, C, F, Fut> AsyncFn<R, I, C> for F
// where
//     F: Fn(R, I, C) -> Fut,
//     Fut: Future,
// {
//     type Fut = Fut;
//     type Output = Fut::Output;
// }

// // type MyAsync<Context> =
// //     AsyncFn<&Request, &Identifiers, Context, , Output = Result<Vec<u8>>>;

// #[derive(Debug)]
// pub enum RouteHandler<AsyncType, Context>
// where
//     AsyncType: for<'a, 'b, 'c> AsyncFn<
//         &'b Request<'a>,
//         &'b Identifiers<'a>,
//         &'c Context,
//         Output = Result<Vec<u8>>,
//     >,
// {
//     Sync(SyncRouteHandler<Context>),
//     Async(AsyncType),
// }

// impl<F, Context: Clone> From<F> for RouteHandler<F, Context>
// where
//     F: for<'a, 'b, 'c> AsyncFn<
//         &'b Request<'a>,
//         &'b Identifiers<'a>,
//         &'c Context,
//         Output = Result<Vec<u8>>,
//     >,
// {
//     fn from(f: F) -> Self {
//         return Self::Async(f);
//     }
// }

// // impl<F, Context: Clone> From<F> for AsyncRouteHandler<Context>
// // where
// //     F: for<'a, 'b> AsyncFn<&'a Request<'a>, &'a Identifiers<'a>, &'b Context, Output = i32>,
// // {
// //     fn from(f: F) -> Self {
// //         return Box;
// //     }
// // }

// struct PlaceholderContext {}

// fn thunk(_req: &Request, _path_vals: &Identifiers, _ctx: &PlaceholderContext) -> Result<Vec<u8>> {
//     return Ok(Vec::new());
// }

// async fn thunk_async(
//     _req: &Request<'_>,
//     _path_vals: &Identifiers<'_>,
//     _ctx: &PlaceholderContext,
// ) -> Result<Vec<u8>> {
//     return Ok(Vec::new());
// }

// async fn thunk_async2(
//     _req: &Request<'_>,
//     _path_vals: &Identifiers<'_>,
//     _ctx: &PlaceholderContext,
// ) -> Result<Vec<u8>> {
//     return Ok(Vec::new());
// }

// fn add_route_async<F, Context>(path: &'static str, handler: F) -> Result<()>
// where
//     F: for<'a, 'b, 'c> AsyncFn<
//         &'b Request<'a>,
//         &'b Identifiers<'a>,
//         &'c Context,
//         Output = Result<Vec<u8>>,
//     >,
// {
//     let p = RouteHandler::Async(handler);

//     todo!();
// }

// fn add_route_sync<Context>(path: &'static str, handler: SyncRouteHandler<Context>) -> Result<()> {
//     let mut vec = Vec::new();
//     let p = RouteHandler::Async(thunk_async);
//     vec.push(p);
//     vec.push(RouteHandler::Async(&thunk_async2));

//     let p = RouteHandler::Sync(handler);
//     vec.push(p);

//     todo!();
// }

// async fn g<F>(y: F) -> Result<Vec<u8>>
// where
//     F: for<'a> AsyncFn<
//         &'a Request<'a>,
//         &'a Identifiers<'a>,
//         &'a PlaceholderContext,
//         Output = Result<Vec<u8>>,
//     >,
// {
//     let buf = [0];
//     let req = Request::from(&buf)?;
//     let ids = Identifiers {
//         path_values: HashMap::new(),
//     };
//     let ctx = PlaceholderContext {};

//     return y(&req, &ids, &ctx).await;
// }

// fn test_add() {
//     add_route_async("path", &thunk_async);
//     add_route_sync("path", thunk);
// }
