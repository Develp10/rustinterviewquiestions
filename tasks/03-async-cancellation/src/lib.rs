//! # Задача 03: Cancellation-safe чтение из TCP-потока
//!
//! Реализуйте read_line_with_timeout — читает строку с таймаутом.
//! BufReader должен жить снаружи функции для cancel-safety.
//!
//! Запуск: cargo test -p async-cancellation

use std::error::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

/// Читает одну строку из reader с таймаутом.
///
/// Cancel-safe: BufReader живёт снаружи — байты не теряются при отмене.
///
/// # Returns
/// - Ok(Some(line)) — прочитана строка (без \n/\r\n)
/// - Ok(None) — EOF (клиент закрыл соединение)
/// - Err(_) — таймаут или IO-ошибка
pub async fn read_line_with_timeout(
    reader: &mut BufReader<TcpStream>,
    timeout: Duration,
) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    let mut line = String::new();

    tokio::select! {
        result = reader.read_line(&mut line) => {
            match result? {
                0 => Ok(None), // EOF
                _ => {
                    // Нормализуем окончания строк
                    if line.ends_with('\n') { line.pop(); }
                    if line.ends_with('\r') { line.pop(); }
                    Ok(Some(line))
                }
            }
        }
        _ = sleep(timeout) => {
            Err("connection timeout".into())
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    use tokio::net::{TcpListener, TcpStream};

    async fn make_pair() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let client = TcpStream::connect(addr).await.unwrap();
        let (server, _) = listener.accept().await.unwrap();
        (client, server)
    }

    #[tokio::test]
    async fn reads_single_line() {
        let (mut client, server) = make_pair().await;
        client.write_all(b"hello\n").await.unwrap();

        let mut reader = BufReader::new(server);
        let line = read_line_with_timeout(&mut reader, Duration::from_secs(1))
            .await
            .unwrap();
        assert_eq!(line, Some("hello".to_string()));
    }

    #[tokio::test]
    async fn detects_eof() {
        let (client, server) = make_pair().await;
        drop(client);

        let mut reader = BufReader::new(server);
        let result = read_line_with_timeout(&mut reader, Duration::from_secs(1))
            .await
            .unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn times_out() {
        let (_client, server) = make_pair().await;
        let mut reader = BufReader::new(server);
        let result =
            read_line_with_timeout(&mut reader, Duration::from_millis(50)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn buffer_preserved_across_calls() {
        let (mut client, server) = make_pair().await;
        client.write_all(b"line1\nline2\n").await.unwrap();

        let mut reader = BufReader::new(server);
        let r1 = read_line_with_timeout(&mut reader, Duration::from_secs(1))
            .await
            .unwrap();
        let r2 = read_line_with_timeout(&mut reader, Duration::from_secs(1))
            .await
            .unwrap();

        assert_eq!(r1, Some("line1".to_string()));
        assert_eq!(r2, Some("line2".to_string()));
    }

    #[tokio::test]
    async fn strips_crlf() {
        let (mut client, server) = make_pair().await;
        client.write_all(b"windows\r\n").await.unwrap();

        let mut reader = BufReader::new(server);
        let line = read_line_with_timeout(&mut reader, Duration::from_secs(1))
            .await
            .unwrap();
        assert_eq!(line, Some("windows".to_string()));
    }
}
