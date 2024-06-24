#[cfg(test)]
mod tests {
    #[derive(Debug)]
    struct PlaceholderContext {}

    use http_routing_rust::callback::asynchronous::AsynchronousFunction;
    use http_routing_rust::callback::synchronous::SynchronousFunction;
    use http_routing_rust::callback::Response;
    use http_routing_rust::http::Request;
    use http_routing_rust::matching::identifiers::Identifiers;

    fn thunk_sync1(_req: &Request, _ids: &Identifiers, _server: &PlaceholderContext) -> Response {
        return Ok(vec![0]);
    }
    fn thunk_sync2(_req: &Request, _ids: &Identifiers, _server: &PlaceholderContext) -> Response {
        return Ok(vec![1]);
    }

    async fn thunk_async1(
        _req: &Request<'_>,
        _ids: &Identifiers<'_>,
        _server: &PlaceholderContext,
    ) -> Response {
        return Ok(vec![0]);
    }
    async fn thunk_async2(
        _req: &Request<'_>,
        _ids: &Identifiers<'_>,
        _server: &PlaceholderContext,
    ) -> Response {
        return Ok(vec![1]);
    }

    const HOME_REQUEST: &[u8; 18] = b"GET / HTTP/1.1\r\n\r\n";

    fn test_defaults() -> (Request<'static>, Identifiers<'static>, PlaceholderContext) {
        (
            Request::try_from(&HOME_REQUEST[..]).unwrap(),
            Identifiers::default(),
            PlaceholderContext {},
        )
    }

    #[test]
    fn storing_sync_functions() {
        let (req, ids, server) = test_defaults();
        let boxed_sync: SynchronousFunction<_> = thunk_sync1.into();

        let resp = boxed_sync.invoke(&req, &ids, &server);
        assert!(resp.is_ok());
        assert_eq!(resp.unwrap(), thunk_sync1(&req, &ids, &server).unwrap());
    }

    #[test]
    fn storing_sync_functions_in_collection() {
        let (req, ids, server) = test_defaults();

        #[allow(clippy::useless_vec)]
        let funcs_sync: Vec<SynchronousFunction<_>> = vec![thunk_sync1.into(), thunk_sync2.into()];

        let resp = funcs_sync[0].invoke(&req, &ids, &server);
        assert!(resp.is_ok());
        assert_eq!(resp.unwrap(), thunk_sync1(&req, &ids, &server).unwrap());

        let resp = funcs_sync[1].invoke(&req, &ids, &server);
        assert!(resp.is_ok());
        assert_eq!(resp.unwrap(), thunk_sync2(&req, &ids, &server).unwrap());

        assert_ne!(
            funcs_sync[0].invoke(&req, &ids, &server).unwrap(),
            funcs_sync[1].invoke(&req, &ids, &server).unwrap()
        );
    }

    #[async_std::test]
    async fn storing_async_functions() {
        let (req, ids, server) = test_defaults();
        let boxed_async: AsynchronousFunction<_> = thunk_async1.into();

        let resp = boxed_async.invoke(&req, &ids, &server).await;
        assert!(resp.is_ok());
        assert_eq!(
            resp.unwrap(),
            thunk_async1(&req, &ids, &server).await.unwrap()
        );
    }
}
