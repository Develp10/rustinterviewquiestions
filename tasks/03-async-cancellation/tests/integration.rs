//! Интеграционные тесты для задачи 03: read_line_with_timeout
//!
//! Запуск: cargo test -p async-cancellation

use async_cancellation::read_line_with_timeout;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::Duration;

async fn make_pair() -> (TcpStream, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let client = TcpStream::connect(addr).await.unwrap();
    let (server, _) = listener.accept().await.unwrap();
    (client, server)
}

#[tokio::test]
async fn integration_reads_multiple_lines() {
    let (mut client, server) = make_pair().await;
    client
        .write_all(b"first\nsecond\nthird\n")
        .await
        .unwrap();
    drop(client); // закрываем после записи

    let mut reader = BufReader::new(server);
    let l1 = read_line_with_timeout(&mut reader, Duration::from_secs(1))
        .await
        .unwrap();
    let l2 = read_line_with_timeout(&mut reader, Duration::from_secs(1))
        .await
        .unwrap();
    let l3 = read_line_with_timeout(&mut reader, Duration::from_secs(1))
        .await
        .unwrap();
    let eof = read_line_with_timeout(&mut reader, Duration::from_secs(1))
        .await
        .unwrap();

    assert_eq!(l1, Some("first".to_string()));
    assert_eq!(l2, Some("second".to_string()));
    assert_eq!(l3, Some("third".to_string()));
    assert_eq!(eof, None);
}

#[tokio::test]
async fn integration_timeout_returns_error() {
    let (_client, server) = make_pair().await;
    let mut reader = BufReader::new(server);
    let result = read_line_with_timeout(&mut reader, Duration::from_millis(30)).await;
    assert!(result.is_err(), "должна быть ошибка таймаута");
}

#[tokio::test]
async fn integration_empty_line_is_preserved() {
    let (mut client, server) = make_pair().await;
    client.write_all(b"\n").await.unwrap();

    let mut reader = BufReader::new(server);
    let line = read_line_with_timeout(&mut reader, Duration::from_secs(1))
        .await
        .unwrap();
    assert_eq!(line, Some(String::new()));
}
