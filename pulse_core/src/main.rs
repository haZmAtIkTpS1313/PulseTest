use reqwest::Client;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::time::{Instant, Duration};
use tokio::time::{interval, timeout};
use tokio::net::TcpListener;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use std::collections::VecDeque;

struct Metrics {
    rps: u64,
    avg_latency_ms: f64,
    p95_latency_ms: f64,
    errors: u64,
}

#[tokio::main]
async fn main() {
    // Конфигурация теста
    let target_url = "https://httpbin.org/get".to_string(); // Тестовый API
    let duration_secs = 60; // Длительность теста
    let concurrent_requests = 10; // Параллельные запросы

    println!("🚀 PulseTest Core запускается...");
    println!("📡 Целевой URL: {}", target_url);
    println!("⏱️  Длительность: {} сек", duration_secs);
    println!("🔥 Параллельных запросов: {}", concurrent_requests);

    // Запускаем TCP сервер для отправки метрик в Qt
    let listener = TcpListener::bind("127.0.0.1:9000").await.unwrap();
    println!("📊 Metrics server listening on port 9000");

    // Счетчики
    let req_count = Arc::new(AtomicU64::new(0));
    let error_count = Arc::new(AtomicU64::new(0));
    let latencies = Arc::new(Mutex::new(VecDeque::with_capacity(1000)));
    let is_running = Arc::new(AtomicBool::new(true));

    // 1. Фоновая задача: генерация реальной нагрузки
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    let req_count_clone = Arc::clone(&req_count);
    let error_count_clone = Arc::clone(&error_count);
    let latencies_clone = Arc::clone(&latencies);
    let is_running_clone = Arc::clone(&is_running);
    let url_clone = target_url.clone();

    tokio::spawn(async move {
        let mut handles = vec![];

        // Запускаем N параллельных задач
        for _ in 0..concurrent_requests {
            let client = client.clone();
            let url = url_clone.clone();
            let req_count = Arc::clone(&req_count_clone);
            let error_count = Arc::clone(&error_count_clone);
            let latencies = Arc::clone(&latencies_clone);
            let is_running = Arc::clone(&is_running_clone);

            let handle = tokio::spawn(async move {
                while is_running.load(Ordering::Relaxed) {
                    let start = Instant::now();
                    
                    match client.get(&url).send().await {
                        Ok(_) => {
                            let latency = start.elapsed().as_millis() as f64;
                            req_count.fetch_add(1, Ordering::Relaxed);
                            
                            let mut lat = latencies.lock().await;
                            lat.push_back(latency);
                            if lat.len() > 1000 {
                                lat.pop_front();
                            }
                        }
                        Err(_) => {
                            error_count.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    
                    // Небольшая пауза, чтобы не перегружать
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            });
            
            handles.push(handle);
        }

        // Останавливаем через duration_secs
        tokio::time::sleep(Duration::from_secs(duration_secs)).await;
        is_running_clone.store(false, Ordering::Relaxed);

        // Ждем завершения всех задач
        for handle in handles {
            let _ = handle.await;
        }

        println!("✅ Тест завершен!");
    });

    // 2. Обработка подключений от Qt UI
    if let Ok((mut socket, _)) = listener.accept().await {
        println!("🔌 Qt UI подключен!");
        let mut ticker = interval(Duration::from_secs(1));
        
        loop {
            ticker.tick().await;
            
            // Считаем метрики за последнюю секунду
            let current_rps = req_count.swap(0, Ordering::Relaxed);
            let current_errors = error_count.swap(0, Ordering::Relaxed);
            
            // Считаем latency
            let mut lat = latencies.lock().await;
            let mut latencies_vec: Vec<f64> = lat.drain(..).collect();
            drop(lat);
            
            let (avg_latency, p95_latency) = if !latencies_vec.is_empty() {
                latencies_vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let avg = latencies_vec.iter().sum::<f64>() / latencies_vec.len() as f64;
                let p95_index = (latencies_vec.len() as f64 * 0.95) as usize;
                let p95 = latencies_vec[p95_index.min(latencies_vec.len() - 1)];
                (avg, p95)
            } else {
                (0.0, 0.0)
            };

            // Формируем JSON для Qt
            let metric = serde_json::json!({
                "rps": current_rps,
                "avg_latency_ms": format!("{:.2}", avg_latency),
                "p95_latency_ms": format!("{:.2}", p95_latency),
                "errors": current_errors
            });
            
            let msg = format!("{}\n", metric.to_string());
            if socket.write_all(msg.as_bytes()).await.is_err() {
                println!("❌ Клиент отключился");
                break;
            }
        }
    }
}