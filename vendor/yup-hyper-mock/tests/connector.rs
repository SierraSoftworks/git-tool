//! Test usage of connectors which work entirely without macros

use env_logger;

use futures::stream::TryStreamExt;

use hyper::{client::Client, header, Body, Response};

use yup_hyper_mock::{HostToReplyConnector, SequentialConnector};

async fn result_to_bytes(res: Response<Body>) -> String {
    let buf = res
        .into_body()
        .try_fold(Vec::new(), |mut buf, bytes| {
            buf.extend_from_slice(bytes.as_ref());
            async move { Ok(buf) }
        })
        .await
        .unwrap();
    String::from_utf8(buf).unwrap()
}

#[tokio::test]
async fn test_sequential_mock() {
    env_logger::try_init().ok();
    let c = SequentialConnector::new(vec![
        "HTTP/1.1 200 OK\r\n\
            Server: BOGUS\r\n\
            \r\n\
            1"
        .to_string(),
        "HTTP/1.1 200 OK\r\n\
            Server: BOGUS\r\n\
            \r\n\
            2"
        .to_string(),
    ]);

    let client = Client::builder().build::<SequentialConnector, Body>(c);

    let res = client
        .get("http://127.0.0.1".parse().unwrap())
        .await
        .unwrap();
    assert_eq!(result_to_bytes(res).await, "1");

    let res = client
        .get("http://127.0.0.1".parse().unwrap())
        .await
        .unwrap();
    assert_eq!(result_to_bytes(res).await, "2");
}

/// Just to test the result of `mock_connector!` - this test was copied from hyper.
#[tokio::test]
async fn test_redirect_followall() {
    env_logger::try_init().ok();
    let mut c = HostToReplyConnector::default();
    c.m.insert(
        "http://127.0.0.1".to_string(),
        "HTTP/1.1 301 Redirect\r\n\
                    Location: http://127.0.0.2\r\n\
                    Server: mock1\r\n\
                    \r\n\
                "
        .to_string(),
    );
    c.m.insert(
        "http://127.0.0.2".to_string(),
        "HTTP/1.1 302 Found\r\n\
                    Location: https://127.0.0.3\r\n\
                    Server: mock2\r\n\
                    \r\n\
                "
        .to_string(),
    );
    c.m.insert(
        "http://127.0.0.3".to_string(),
        "HTTP/1.1 200 OK\r\n\
                    Server: mock3\r\n\
                    \r\n\
                    "
        .to_string(),
    );

    let client = Client::builder().build(c);

    async fn check_server(client: &Client<HostToReplyConnector, Body>, url: &str, server: &str) {
        let res = client.get(url.parse().unwrap()).await.unwrap();
        let header = header::HeaderValue::from_str(&server).unwrap();
        assert_eq!(res.headers().get(header::SERVER), Some(&header));
    }
    check_server(&client, "http://127.0.0.1", "mock1").await;
    check_server(&client, "http://127.0.0.2", "mock2").await;
    check_server(&client, "http://127.0.0.3", "mock3").await;
}
