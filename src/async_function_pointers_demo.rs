//! This application is a proof on concept meant to demo seemless handling between synchronous
//! and asynchronous code
#![allow(warnings)]

use core::pin::Pin;
use std::future::Future;

type Res = Result<(), Box<dyn std::error::Error>>;

#[derive(Clone)]
struct SyncPointer<'a> {
    pub func: fn(msg: &'a str) -> Res,
}

/// Type to encapsulate asynchronous function pointers

struct AsyncFnPointer<'a> {
    pub func: Box<dyn FnOnce(&'a str) -> Pin<Box<dyn Future<Output = Res>>>>,
}

// impl<'a> Clone for AsyncFnPointer<'a> {
//     fn clone(&self) -> Self {
//         Self {
//             func: self.func.clone(),
//         }
//     }
// }

enum FunctionPointer<'a> {
    Sync(SyncPointer<'a>),
    Async(AsyncFnPointer<'a>),
}

fn sync_print(msg: &str) -> Res {
    println!("Sync: {msg}");
    return Ok(());
}

async fn async_print(msg: &str) -> Res {
    println!("Async: {msg}");
    return Ok(());
}

async fn call<'a>(msg: &'a str, f: FunctionPointer<'a>) -> Res {
    // let msg = "static message";
    match f {
        FunctionPointer::Sync(g) => (g.func)(msg),
        FunctionPointer::Async(h) => (h.func)(msg).await,
    }
}

fn convert_fn<'a, 'b, T>(f: fn(&'a str) -> T) -> FunctionPointer<'b>
where
    T: Future<Output = Res> + 'static,
{
    FunctionPointer::Async(AsyncFnPointer {
        func: Box::new(move |n| Box::pin(f(n))),
    })
}

#[async_std::main]
async fn main() {
    let sync = FunctionPointer::Sync(SyncPointer { func: sync_print });
    let a_sync = convert_fn(async_print);
    let msg = String::from("Hello World");
    // let msg = "static message";

    call(&msg, sync).await;
    call(&msg, a_sync).await;

    println!("Main Finished");
}
