#![allow(unused)] // temporary

use anyhow::Result;
use core::future::Future;
use core::pin::Pin;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use crate::core::request::Request;
pub struct Identifiers<'a> {
    pub path_values: HashMap<&'static str, &'a str>,
}

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

pub type SyncRouteHandler<'a, Context: 'static> = fn(
    req: &'a Request<'a>,
    path_vals: &'a Identifiers<'a>,
    ctx: &'static Context,
) -> Result<Vec<u8>>;

pub type AsyncRouteHandler<Context> =
    fn(req: &Request, path_vals: &Identifiers, ctx: &Context) -> BoxFuture<Result<Vec<u8>>>;

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

// type MyAsync<Context> =
//     AsyncFn<&Request, &Identifiers, Context, , Output = Result<Vec<u8>>>;

// impl<F, Context: Clone> From<F> for AsyncRouteHandler<Context>
// where
//     F: for<'a, 'b> AsyncFn<&'a Request<'a>, &'a Identifiers<'a>, &'b Context, Output = i32>,
// {
//     fn from(f: F) -> Self {
//         return Box;
//     }
// }

struct PlaceholderContext {}

fn thunk(_req: &Request, _path_vals: &Identifiers, _ctx: &PlaceholderContext) -> Result<Vec<u8>> {
    return Ok(Vec::new());
}

async fn thunk_async(
    _req: &Request<'_>,
    _path_vals: &Identifiers<'_>,
    _ctx: &'static PlaceholderContext,
) -> Result<Vec<u8>> {
    return Ok(Vec::new());
}

async fn thunk_async2(
    _req: &Request<'_>,
    _path_vals: &Identifiers<'_>,
    _ctx: &'static PlaceholderContext,
) -> Result<Vec<u8>> {
    return Ok(Vec::new());
}

type BoxedAsync<'a, Context> = Rc<
    dyn FnOnce(
        &'a Request<'a>,
        &'a Identifiers<'a>,
        &'static Context,
    ) -> Pin<Rc<dyn Future<Output = Result<Vec<u8>>>>>,
>;

#[derive(Clone)]
struct AsyncHandler<'a, Context: 'static> {
    pub func: BoxedAsync<'a, Context>,
}

fn force_boxed<'a, Context: 'static, F, T>(f: F) -> AsyncHandler<'a, Context>
where
    F: 'static + Fn(&'a Request<'a>, &'a Identifiers<'a>, &'static Context) -> T,
    T: Future<Output = Result<Vec<u8>>> + 'static,
{
    return AsyncHandler {
        func: Rc::new(move |r, p, c| {
            return Rc::pin(f(r, p, c));
        }),
    };
}

// #[derive(Debug)]
pub enum RouteHandler<'a, Context: 'static> {
    Sync(SyncRouteHandler<'a, Context>),
    Async(AsyncHandler<'a, Context>),
}

async fn add_route_async<'a, Context: 'static, F, T>(path: &'static str, handler: F) -> Result<()>
where
    F: 'static + Fn(&'a Request<'a>, &'a Identifiers<'a>, &'static Context) -> T,
    T: Future<Output = Result<Vec<u8>>> + 'static,
{
    let p = RouteHandler::Async(force_boxed(handler));
    todo!();
}

fn add_route_sync(path: &'static str, handler: SyncRouteHandler<PlaceholderContext>) -> Result<()> {
    let mut vec: Vec<RouteHandler<PlaceholderContext>> = Vec::new();
    let p = force_boxed(thunk_async);
    let p = RouteHandler::Async(p);
    vec.push(p);
    vec.push(RouteHandler::Async(force_boxed(thunk_async2)));

    let p = RouteHandler::Sync(handler);
    vec.push(p);

    todo!();
}

fn test_add() {
    let _ = add_route_async("path", thunk_async);
    let _ = add_route_sync("path", thunk);
}

async fn inc(src: &u32) -> u32 {
    src + 1
}

async fn inc2(src: &u32) -> u32 {
    src + 2
}

type Incrementer<'a> = Box<dyn Fn(&'a u32) -> Pin<Box<dyn Future<Output = u32>>>>;

struct IncrementStruct<'a> {
    pub f: Incrementer<'a>,
}

fn force_boxed2<'a, F, T>(f: F) -> IncrementStruct<'a>
where
    F: 'static + Fn(&'a u32) -> T,
    T: Future<Output = u32> + 'static,
{
    return IncrementStruct {
        f: Box::new(move |n| {
            return Box::pin(f(&Box::new(n)));
        }),
    };
}

async fn increment_printer<'a>(inc: Incrementer<'a>) {
    println!("{}", inc(&1).await);
}

async fn passthru(f: &IncrementStruct<'_>) {
    println!("{}", (f.f)(&1).await);
}

#[async_std::test]
async fn test_main() {
    let vec = vec![force_boxed2(inc), force_boxed2(inc2)];
    for func in &vec {
        passthru(func).await;
    }
    for func in &vec {
        passthru(func).await;
    }
    return;
}

type Pointer<'a, 'b> = fn(msg: &'a str) -> &'b str;

#[derive(Clone, Copy)]
struct FnPointer<'a, 'b>
where
    'a: 'b,
{
    pub func: Pointer<'a, 'b>,
}

fn first_word(sentence: &str) -> &str {
    let mut space = None;
    for (i, c) in sentence.chars().enumerate() {
        if c == ' ' {
            space = Some(i);
            break;
        }
    }
    match space {
        None => return sentence,
        Some(i) => return &sentence[0..i],
    }
}

fn exclude_first_word(sentence: &str) -> &str {
    let mut space = None;
    for (i, c) in sentence.chars().enumerate() {
        if c == ' ' {
            space = Some(i);
            break;
        }
    }
    match space {
        None => return &sentence[sentence.len()..],
        Some(i) => return &sentence[i..],
    }
}

fn apply_pointer(f: FnPointer) {
    let sentences = vec!["potato1 potato2", "aojsdhfouanwsdf;kjl", "hi world"];

    for s in sentences {
        println!("{} -> {}", s, (f.func)(s));
    }
}

#[test]
fn trying_pointer() {
    let pointers = vec![
        FnPointer { func: first_word },
        FnPointer {
            func: exclude_first_word,
        },
    ];
    for func in pointers {
        apply_pointer(func);
    }
}

type BoxedAsyncPointer<'a, 'b> = Box<dyn Fn(&'a str) -> Pin<Box<dyn Future<Output = &'b str>>>>;

struct AsyncFnPointer<'a, 'b>
where
    'a: 'b,
{
    pub func: BoxedAsyncPointer<'a, 'b>,
}

use async_std::task;
use std::time::Duration;

async fn async_first_word<'a, 'b>(sentence: &'a str) -> &'b str
where
    'a: 'b,
{
    let mut space = None;
    for (i, c) in sentence.chars().enumerate() {
        if c == ' ' {
            space = Some(i);
            break;
        }
    }
    task::sleep(Duration::from_secs(1)).await;
    match space {
        None => return sentence,
        Some(i) => return &sentence[0..i],
    }
}

async fn async_exclude_first_word(sentence: &str) -> &str {
    let mut space = None;
    for (i, c) in sentence.chars().enumerate() {
        if c == ' ' {
            space = Some(i);
            break;
        }
    }
    task::sleep(Duration::from_secs(1)).await;
    match space {
        None => return &sentence[sentence.len()..],
        Some(i) => return &sentence[i..],
    }
}

async fn async_apply_pointer(f: AsyncFnPointer<'_, '_>) {
    let sentences = vec!["potato potato", "aojsdhfouanwsdf;kjl", "hi world today"];

    for s in sentences {
        println!("{} -> {}", s, (f.func)(s).await);
    }
}

async fn double_pass(f: AsyncFnPointer<'_, '_>) {
    let sentences = vec!["ffff gggg", "l l l l", "a a a a"];

    for s in sentences {
        println!("{} -> {}", s, (f.func)(s).await);
    }
    async_apply_pointer(f).await;
}

pub fn force_asyncboxed<'a, 'b, F, T>(f: F) -> BoxedAsyncPointer<'a, 'b>
where
    'a: 'b,
    F: 'static + Fn(&'a str) -> T,
    T: Future<Output = &'b str> + 'static,
{
    return Box::new(move |s| {
        return Box::pin(f(s));
    });
}

#[async_std::test]
async fn async_trying_pointer() {
    let pointers = vec![
        AsyncFnPointer {
            func: force_asyncboxed(async_first_word),
        },
        AsyncFnPointer {
            func: force_asyncboxed(async_exclude_first_word),
        },
    ];
    for func in pointers {
        double_pass(func).await;
    }
}

enum SyncOrAsync<'a, 'b>
where
    'a: 'b,
{
    Sync(FnPointer<'a, 'b>),
    Async(AsyncFnPointer<'a, 'b>),
}

#[async_std::test]
async fn collect_pointer() {
    let pointers = vec![
        SyncOrAsync::Async(AsyncFnPointer {
            func: force_asyncboxed(async_first_word),
        }),
        SyncOrAsync::Async(AsyncFnPointer {
            func: force_asyncboxed(async_exclude_first_word),
        }),
        SyncOrAsync::Sync(FnPointer {
            func: exclude_first_word,
        }),
        SyncOrAsync::Sync(FnPointer { func: first_word }),
    ];

    for e in pointers {
        match e {
            SyncOrAsync::Async(f) => async_apply_pointer(f).await,
            SyncOrAsync::Sync(f) => apply_pointer(f),
        }
    }
    return;
}
