use crate::matching::identifiers::Identifiers;
use crate::prelude::*;

type Handler<'a, Context> = dyn Fn(&'a Request<'a>, &'a Identifiers<'a>, &Context) -> Response;

pub struct SynchronousFunction<'a, Context: 'static> {
    callback: Box<Handler<'a, Context>>,
}

impl<'a, 'b, Context: 'static> SynchronousFunction<'b, Context>
where
    'a: 'b,
{
    pub fn invoke(
        &self,
        req: &'a Request<'a>,
        ids: &'a Identifiers<'a>,
        server: &Context,
    ) -> Response {
        return (self.callback)(req, ids, server);
    }
}

impl<'a, Context: 'static, F> From<F> for SynchronousFunction<'a, Context>
where
    F: Fn(&'a Request<'a>, &'a Identifiers<'a>, &Context) -> Response + 'static,
{
    fn from(value: F) -> Self {
        return Self {
            callback: Box::new(value),
        };
    }
}
