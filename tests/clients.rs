extern crate rsocket_rust;

use rsocket_rust::prelude::*;

#[tokio::main]
#[test]
async fn test() {
    let cli = RSocketFactory::connect()
        .acceptor(|| Box::new(EchoRSocket))
        .transport(URI::Tcp("127.0.0.1:7878".to_string()))
        .setup(Payload::from("READY!"))
        .mime_type("text/plain", "text/plain")
        .start()
        .await
        .unwrap();
    let req = Payload::builder()
        .set_data_utf8("Hello World!")
        .set_metadata_utf8("Rust")
        .build();
    let res = cli.request_response(req).await.unwrap();
    println!("got: {:?}", res);
    cli.close();
}

#[tokio::main]
#[test]
async fn test_client() {
    env_logger::builder().init();
    let cli = RSocketFactory::connect()
        .acceptor(|| Box::new(EchoRSocket))
        .transport(URI::Tcp("127.0.0.1:7878".to_string()))
        .setup(Payload::from("READY!"))
        .mime_type("text/plain", "text/plain")
        .start()
        .await
        .unwrap();
    exec_metadata_push(&cli).await;
    exec_fire_and_forget(&cli).await;
    exec_request_response(&cli).await;
    exec_request_stream(&cli).await;
    cli.close();
}

async fn exec_request_response(socket: &Client) {
    // request response
    let sending = Payload::builder()
        .set_data_utf8("Hello World!")
        .set_metadata_utf8("I Rust!")
        .build();
    let result = socket.request_response(sending).await.unwrap();
    println!("REQUEST_RESPONSE: {:?}", result);
}

async fn exec_metadata_push(socket: &Client) {
    let pa = Payload::builder().set_metadata_utf8("Hello World!").build();
    // metadata push
    socket.metadata_push(pa).await.unwrap();
}

async fn exec_fire_and_forget(socket: &Client) {
    // request fnf
    let fnf = Payload::from("Hello World!");
    socket.fire_and_forget(fnf).await.unwrap();
}

async fn exec_request_stream(socket: &Client) {
    // request stream
    let sending = Payload::builder()
        .set_data_utf8("Hello Rust!")
        .set_metadata_utf8("foobar")
        .build();

    let mut results = socket.request_stream(sending);
    loop {
        match results.next().await {
            Some(v) => println!("STREAM_RESPONSE: {:?}", v.unwrap()),
            None => break,
        }
    }
}
