use hyper;
#[macro_use]
extern crate yup_hyper_mock;

use futures::stream::TryStreamExt;

use hyper::{client::Client, header, Body, Response};

mock_connector_in_order! (MockSequential {
                                  "HTTP/1.1 200 OK\r\n\
                                 Server: BOGUS\r\n\
                                 \r\n\
                                 1"

                                 "HTTP/1.1 200 OK\r\n\
                                 Server: BOGUS\r\n\
                                 \r\n\
                                 2"
                                  });

mock_connector!(MockRedirectPolicy {
    "http://127.0.0.1" =>       "HTTP/1.1 301 Redirect\r\n\
                                 Location: http://127.0.0.2\r\n\
                                 Server: mock1\r\n\
                                 \r\n\
                                "
    "http://127.0.0.2" =>       "HTTP/1.1 302 Found\r\n\
                                 Location: https://127.0.0.3\r\n\
                                 Server: mock2\r\n\
                                 \r\n\
                                "
    "https://127.0.0.3" =>      "HTTP/1.1 200 OK\r\n\
                                 Server: mock3\r\n\
                                 \r\n\
                                "
});

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

/// Just to test the result of `mock_connector!` - this test was copied from hyper.
#[tokio::test]
async fn test_redirect_followall() {
    let client = Client::builder().build::<MockRedirectPolicy, Body>(MockRedirectPolicy::default());

    async fn check_server(client: &Client<MockRedirectPolicy, Body>, url: &str, server: &str) {
        let res = client.get(url.parse().unwrap()).await.unwrap();
        let header = header::HeaderValue::from_str(&server).unwrap();
        assert_eq!(res.headers().get(header::SERVER), Some(&header));
        let _ = result_to_bytes(res).await;
    }
    check_server(&client, "http://127.0.0.1", "mock1").await;
    check_server(&client, "http://127.0.0.2", "mock2").await;
    check_server(&client, "https://127.0.0.3", "mock3").await;
}

#[tokio::test]
async fn test_sequential_mock() {
    let client = Client::builder().build::<MockSequential, Body>(MockSequential::default());

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
