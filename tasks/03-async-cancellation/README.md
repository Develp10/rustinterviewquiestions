## 03. Cancellation-safe чтение из TCP-потока

**Уровень:** Senior  
**Темы:** `tokio::select!`, cancel safety, `Pin`, `AsyncRead`, `BufReader`

---

### Условие

Есть TCP-сервер, который читает сообщения из потока построчно. Нужно добавить таймаут: если клиент не прислал строку за N секунд — разорвать соединение. Реализуйте функцию `read_line_with_timeout`.

```rust
use tokio::net::TcpStream;
use tokio::time::Duration;

/// Читает одну строку из stream.
/// Возвращает Ok(None) при разрыве соединения.
/// Возвращает Err при таймауте или IO-ошибке.
pub async fn read_line_with_timeout(
    stream: &mut TcpStream,
    timeout: Duration,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    todo!()
}
```

Бонус: объясните, почему `read_exact` нельзя использовать внутри `select!` и что теряется при отмене.

---

### Наивная попытка

```rust
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::time::{timeout, Duration};

pub async fn read_line_naive(stream: &mut TcpStream, dur: Duration)
    -> Result<Option<String>, Box<dyn std::error::Error>>
{
    let mut reader = BufReader::new(stream); // создаём на каждый вызов!
    let mut line = String::new();
    // Проблема: если timeout сработает на середине строки,
    // байты уже вычитаны из ядра и потеряны навсегда.
    match timeout(dur, reader.read_line(&mut line)).await {
        Ok(Ok(0)) => Ok(None),
        Ok(Ok(_)) => Ok(Some(line)),
        Ok(Err(e)) => Err(e.into()),
        Err(_) => Err("timeout".into()),
    }
}
```

Проблема в том, что `BufReader` создаётся внутри функции. При drop буфер теряется, а байты, уже считанные из ядра в буфер, — вместе с ним.

---

### Разбор

**Cancel safety** означает: если future дропнуть на любом `.await`, внешнее состояние (TCP-сокет, файл) останется консистентным. `read_line` из `AsyncBufReadExt` — cancel-safe только если BufReader живёт между вызовами. `read_exact` — не cancel-safe никогда: байты уже вычитаны, их не вернуть.

**Правильный паттерн:** `BufReader` должен жить снаружи функции и передаваться по `&mut`. Тогда при отмене буфер сохраняется и следующий вызов продолжит с того же места.

**Как работает select с Pin:** futures в `select!` должны быть запинены, иначе их нельзя хранить между poll-вызовами. Макрос делает это автоматически через `tokio::pin!`.

---

### Рабочее решение

```rust
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

// BufReader живёт снаружи — байты не теряются при отмене
pub async fn read_line_with_timeout(
    reader: &mut BufReader<TcpStream>,
    timeout: Duration,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let mut line = String::new();
    
    tokio::select! {
        result = reader.read_line(&mut line) => {
            match result? {
                0 => Ok(None),                // EOF
                _ => {
                    // Убираем \n и \r\n
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

// Пример использования в сервере
pub async fn handle_client(stream: TcpStream) {
    let mut reader = BufReader::new(stream);
    let timeout = Duration::from_secs(30);
    
    loop {
        match read_line_with_timeout(&mut reader, timeout).await {
            Ok(Some(line)) => {
                println!("Got: {line}");
            }
            Ok(None) => {
                println!("Client disconnected");
                break;
            }
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
        }
    }
}
```

---

### Тесты

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
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
    async fn reads_line() {
        let (mut client, server) = make_pair().await;
        client.write_all(b"hello\n").await.unwrap();
        
        let mut reader = BufReader::new(server);
        let line = read_line_with_timeout(&mut reader, Duration::from_secs(1))
            .await.unwrap();
        assert_eq!(line, Some("hello".to_string()));
    }

    #[tokio::test]
    async fn detects_eof() {
        let (client, server) = make_pair().await;
        drop(client); // Закрываем соединение
        
        let mut reader = BufReader::new(server);
        let result = read_line_with_timeout(&mut reader, Duration::from_secs(1))
            .await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn times_out() {
        let (_client, server) = make_pair().await;
        // client жив, но ничего не шлёт
        
        let mut reader = BufReader::new(server);
        let result = read_line_with_timeout(&mut reader, Duration::from_millis(50)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn buffers_preserved_across_calls() {
        let (mut client, server) = make_pair().await;
        // Шлём две строки
        client.write_all(b"line1\nline2\n").await.unwrap();
        
        let mut reader = BufReader::new(server);
        let r1 = read_line_with_timeout(&mut reader, Duration::from_secs(1)).await.unwrap();
        let r2 = read_line_with_timeout(&mut reader, Duration::from_secs(1)).await.unwrap();
        assert_eq!(r1, Some("line1".to_string()));
        assert_eq!(r2, Some("line2".to_string()));
    }
}
```

---

### Что проверять в ревью

- `BufReader` передаётся по `&mut`, а не создаётся внутри функции
- `select!` использует `sleep` (cancel-safe), а не `timeout` с оборачиванием
- Строка очищается от `\r\n` перед возвратом
- Ошибка таймаута типизирована отдельно от IO-ошибки

---

### Cancel safety в стандартных примитивах tokio

| Примитив | Cancel-safe |
|---|---|
| `mpsc::Receiver::recv` | Да |
| `AsyncBufReadExt::read_line` | Да (если BufReader снаружи) |
| `AsyncReadExt::read` | Да |
| `AsyncReadExt::read_exact` | Нет |
| `AsyncWriteExt::write_all` | Нет |
| `Mutex::lock` (tokio) | Да |
| `sleep` | Да |

---

### Ссылки

- [tokio cancel safety docs](https://docs.rs/tokio/latest/tokio/macro.select.html#cancellation-safety)  
- [AsyncBufReadExt](https://docs.rs/tokio/latest/tokio/io/trait.AsyncBufReadExt.html)
