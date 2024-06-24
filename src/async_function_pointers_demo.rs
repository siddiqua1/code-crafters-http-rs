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
// #[derive(Clone)]
struct SimpleCallBack<'req, 'resp>
where
    'req: 'resp,
{
    callback: Box<dyn Fn(Request<'req>) -> BoxFuture<'resp, Res> + 'resp>,
}

// impl<'req, 'resp> Clone for SimpleCallBack<'req, 'resp>
// where
//     'req: 'resp,
// {
//     fn clone(&self) -> Self {
//         return Self {
//             callback: self.callback.clone(),
//         };
//     }
// }

impl<'req, 'resp> SimpleCallBack<'req, 'resp>
where
    'req: 'resp,
{
    fn new<T, F>(f: F) -> Self
    where
        T: Future<Output = Res> + Send + 'resp,
        F: Fn(Request<'req>) -> T + 'resp,
    {
        SimpleCallBack {
            callback: Box::new(move |msg| Box::pin(f(msg))),
        }
    }

    async fn invoke(&self, x: Request<'req>) -> Res {
        return (self.callback)(x).await;
    }
}

async fn foo_request(msg: Request<'_>) -> Res {
    println!("Foo: {msg:?}");
    Ok(())
}

async fn bar_request(msg: Request<'_>) -> Res {
    println!("Bar: {msg:?}");
    Err("Potato".into())
}

const REQUEST_01: &[u8; 21] = b"GET /one HTTP/1.1\r\n\r\n";
const REQUEST_02: &[u8; 21] = b"GET /two HTTP/1.1\r\n\r\n";
fn move_it<T>(_: T) {}

async fn async_with_structs_with_lifetimes() {
    // must define first so that these get dropped after the callbacks
    let r1 = Request::try_from(&REQUEST_01[..]).unwrap();
    let r2 = Request::try_from(&REQUEST_02[..]).unwrap();

    let mut v: Vec<SimpleCallBack> = Vec::new();
    v.push(SimpleCallBack::new(bar_request));
    v.push(SimpleCallBack::new(foo_request));

    v[0].invoke(r1).await;
    v[0].invoke(r2).await;
    // need to consume since async makes taking in a reference a pain
    let r1 = Request::try_from(&REQUEST_01[..]).unwrap();
    let r2 = Request::try_from(&REQUEST_02[..]).unwrap();

    v[1].invoke(r1).await;
    v[1].invoke(r2).await;

    let print_incr = Box::new(move |x| {
        println!("{:?}", x);
        bar_request(x);
    });
    // move_it(v[0].clone());
    move_it(print_incr.clone());
    // vec![test_closure, test_closure.clone()];
}

//-------------------------------------------------------------------------------------------

#[async_std::main]
async fn main() {
    pointer_work_if_no_ref_params().await;
    async_with_ref_params().await;
    async_with_structs_with_lifetimes().await;

    println!("Main Finished");
}
