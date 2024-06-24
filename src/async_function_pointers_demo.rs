//! This application is a proof on concept meant to demo seemless handling between synchronous
//! and asynchronous code
//!
//! From this it seems that the function must consume the underlying request buffer
#![allow(warnings)]

use core::pin::Pin;
use futures::{future::BoxFuture, Future};

type Res = Result<(), Box<dyn std::error::Error>>;

//-------------------------------------------------------------------------------------------
// no reference args in params

async fn foo(msg: String) -> Res {
    println!("{msg}");
    Ok(())
}

async fn bar(msg: String) -> Res {
    println!("{msg}");
    Err("Potato".into())
}

fn wrapper<T, F>(f: F) -> Box<dyn Fn(String) -> BoxFuture<'static, Res>>
where
    T: Future<Output = Res> + Send + 'static,
    F: Fn(String) -> T + 'static,
{
    Box::new(move |msg| Box::pin(f(msg)))
}

type Wrapped = Box<dyn Fn(String) -> BoxFuture<'static, Res>>;

async fn pointer_work_if_no_ref_params() {
    let mut v: Vec<Wrapped> = Vec::new();
    v.push(wrapper(foo));
    v.push(wrapper(bar));

    let args = vec!["argument1", "argument2"];

    for (f, arg) in v.iter().zip(args.iter()) {
        println!("{:?}", f(arg.to_string()).await);
    }
    for (f, arg) in v.iter().zip(args.iter()) {
        println!("{:?}", f(arg.to_string()).await);
    }

    let args = vec!["argument1", "argument2", "argument3"];

    for msg in args {
        for f in &v {
            let _ = f(msg.to_string()).await;
        }
    }
}

//-------------------------------------------------------------------------------------------
// args are refs
struct Caller<'a, 'b>
where
    'a: 'b,
{
    callback: Box<dyn Fn(&'a str) -> BoxFuture<'static, Res> + 'b>,
}
impl<'a, 'b> Caller<'a, 'b>
where
    'a: 'b,
{
    fn new<T, F>(f: F) -> Self
    where
        T: Future<Output = Res> + Send + 'static,
        F: Fn(&'a str) -> T + 'b,
    {
        Caller {
            callback: Box::new(move |msg| Box::pin(f(msg))),
        }
    }

    async fn invoke(&self, x: &'a str) -> Res {
        return (self.callback)(x).await;
    }
}

async fn foo_ref(msg: &str) -> Res {
    println!("Foo: {msg}");
    Ok(())
}

async fn bar_ref(msg: &str) -> Res {
    println!("Bar: {msg}");
    Err("Potato".into())
}

async fn async_with_ref_params() {
    let mut v: Vec<Caller> = Vec::new();
    v.push(Caller::new(foo_ref));
    v.push(Caller::new(bar_ref));

    let args = vec!["argument1", "argument2", "argument3"];

    for msg in args {
        for f in &v {
            let _ = f.invoke(msg).await;
        }
    }
}

//-------------------------------------------------------------------------------------------
// args are refs + the struct ref has a lifetime as well
// need to be able to clone the function pointer

use http_routing_rust::http::Request;

struct SimpleCallBack<'borrow, 'req, 'resp>
where
    'borrow: 'resp, // borrowed data must live atleast as long the time it takes to drive the Future to completion
    'req: 'resp,
    'req: 'borrow,
{
    callback: Box<dyn Fn(&'borrow Request<'req>) -> BoxFuture<'resp, Res> + 'resp>,
}

impl<'borrow, 'req, 'resp> SimpleCallBack<'borrow, 'req, 'resp>
where
    'borrow: 'resp,
{
    fn new<T, F>(f: F) -> Self
    where
        T: Future<Output = Res> + Send + 'resp,
        F: Fn(&'borrow Request<'req>) -> T + 'resp,
    {
        SimpleCallBack {
            callback: Box::new(move |msg| Box::pin(f(msg))),
        }
    }

    async fn invoke(&self, x: &'borrow Request<'req>) -> Res {
        return (self.callback)(x).await;
    }
}

async fn foo_request(msg: &Request<'_>) -> Res {
    println!("Foo: {msg:?}");
    Ok(())
}

async fn bar_request(msg: &Request<'_>) -> Res {
    println!("Bar: {msg:?}");
    Err("Potato".into())
}

const REQUEST_01: &[u8; 21] = b"GET /one HTTP/1.1\r\n\r\n";
const REQUEST_02: &[u8; 21] = b"GET /two HTTP/1.1\r\n\r\n";
const REQUEST_03: &[u8; 23] = b"GET /three HTTP/1.1\r\n\r\n";

async fn async_with_ref_params_and_structs_with_lifetimes() {
    // must define first so that these get dropped after the callbacks
    let args = vec![
        Request::try_from(&REQUEST_01[..]).unwrap(),
        Request::try_from(&REQUEST_02[..]).unwrap(),
        Request::try_from(&REQUEST_03[..]).unwrap(),
    ];

    let mut v: Vec<SimpleCallBack> = Vec::new();
    v.push(SimpleCallBack::new(bar_request));
    v.push(SimpleCallBack::new(foo_request));

    for msg in args.iter() {
        for f in &v {
            let _ = f.invoke(msg).await;
        }
    }
}

async fn same_test_but_callback_passed_in(cb: &SimpleCallBack<'_, '_, '_>) {
    let args = vec![
        Request::try_from(&REQUEST_01[..]).unwrap(),
        Request::try_from(&REQUEST_02[..]).unwrap(),
        Request::try_from(&REQUEST_03[..]).unwrap(),
    ];
    // cb.invoke(&args[0]).await;
}

async fn reversing_order_of_creation_test() {
    todo!()
}

use futures_util::FutureExt;
use http_routing_rust::callback::Response;
async fn bar_request_consume(msg: Request<'_>) -> Response {
    println!("Bar: {msg:?}");
    Err("Potato".into())
}
async fn using_future_ext_shared() {
    let pointer = Box::new(move |msg| bar_request_consume(msg).shared());

    let funcs = vec![pointer.clone(), pointer.clone(), pointer.clone()];
    let args = vec![
        Request::try_from(&REQUEST_01[..]).unwrap(),
        Request::try_from(&REQUEST_02[..]).unwrap(),
        Request::try_from(&REQUEST_03[..]).unwrap(),
    ];

    for msg in args {
        for f in &funcs {
            let _ = (f.clone())(msg.clone()).await;
        }
    }
}

//-------------------------------------------------------------------------------------------

#[async_std::main]
async fn main() {
    println!();
    pointer_work_if_no_ref_params().await;
    println!();
    async_with_ref_params().await;
    println!();
    async_with_ref_params_and_structs_with_lifetimes().await;
    println!();
    using_future_ext_shared().await;
    println!("Main Finished");
}
