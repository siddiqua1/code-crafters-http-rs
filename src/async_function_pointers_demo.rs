//! This application is a proof on concept meant to demo seemless handling between synchronous
//! and asynchronous code
//!
//! From this it seems that the function must consume the underlying request buffer
#![allow(warnings)]

use core::pin::Pin;
use futures::{future::BoxFuture, Future};

type Res = Result<(), Box<dyn std::error::Error>>;

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
}

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

#[async_std::main]
async fn main() {
    pointer_work_if_no_ref_params().await;
    async_with_ref_params().await;

    println!("Main Finished");
}
