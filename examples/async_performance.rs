//! Async performance benchmark for firo_logger.

use firo_logger::{init, log_info, LogLevel, LoggerConfig};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Async Performance Benchmark ===");
    println!("This demo compares sync vs async logging performance.\n");

    // Test parameters
    let num_threads = 4;
    let messages_per_thread = 10_000;
    let total_messages = num_threads * messages_per_thread;

    println!("Test configuration:");
    println!("  - Threads: {}", num_threads);
    println!("  - Messages per thread: {}", messages_per_thread);
    println!("  - Total messages: {}", total_messages);
    println!();

    // Test 1: Synchronous logging
    println!("🔄 Testing synchronous logging...");
    let sync_duration = benchmark_sync_logging(num_threads, messages_per_thread)?;

    println!("✅ Sync test completed in {:.2?}", sync_duration);
    println!(
        "   Throughput: {:.0} messages/second",
        total_messages as f64 / sync_duration.as_secs_f64()
    );
    println!();

    // Small delay between tests
    thread::sleep(Duration::from_millis(500));

    // Test 2: Asynchronous logging
    println!("⚡ Testing asynchronous logging...");
    let async_duration = benchmark_async_logging(num_threads, messages_per_thread)?;

    println!("✅ Async test completed in {:.2?}", async_duration);
    println!(
        "   Throughput: {:.0} messages/second",
        total_messages as f64 / async_duration.as_secs_f64()
    );
    println!();

    // Performance comparison
    println!("📊 Performance Comparison:");
    println!(
        "   Sync:  {:.2?} ({:.0} msg/s)",
        sync_duration,
        total_messages as f64 / sync_duration.as_secs_f64()
    );
    println!(
        "   Async: {:.2?} ({:.0} msg/s)",
        async_duration,
        total_messages as f64 / async_duration.as_secs_f64()
    );

    let speedup = sync_duration.as_secs_f64() / async_duration.as_secs_f64();
    if speedup > 1.0 {
        println!("   🚀 Async is {:.1}x faster than sync!", speedup);
    } else {
        println!("   📊 Sync is {:.1}x faster than async", 1.0 / speedup);
    }

    println!("\nNote: Results may vary based on system load and hardware.");
    println!("Async logging is typically faster for high-volume logging scenarios.");

    Ok(())
}

fn benchmark_sync_logging(
    num_threads: usize,
    messages_per_thread: usize,
) -> Result<Duration, Box<dyn std::error::Error>> {
    // Configure synchronous logging
    let config = LoggerConfig::builder()
        .level(LogLevel::Info)
        .console(false) // Disable console to focus on logging overhead
        .file("benchmark_sync.log")
        .async_logging(0) // Disable async
        .include_caller(false) // Minimize overhead
        .include_thread(false)
        .datetime_format("%H:%M:%S%.3f")
        .metadata("benchmark", "sync")
        .build();

    init(config)?;

    let barrier = Arc::new(Barrier::new(num_threads + 1));
    let mut handles = Vec::new();

    // Start timing threads
    for thread_id in 0..num_threads {
        let barrier_clone = Arc::clone(&barrier);
        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();

            // Log messages
            for i in 0..messages_per_thread {
                if let Err(_) = log_info!("Sync benchmark message {} from thread {}", i, thread_id)
                {
                    break; // Stop on error
                }
            }
        });
        handles.push(handle);
    }

    // Start the benchmark
    let start = Instant::now();
    barrier.wait(); // Release all threads

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();

    // Give sync logging time to flush
    thread::sleep(Duration::from_millis(100));

    Ok(duration)
}

fn benchmark_async_logging(
    num_threads: usize,
    messages_per_thread: usize,
) -> Result<Duration, Box<dyn std::error::Error>> {
    // Configure asynchronous logging
    let config = LoggerConfig::builder()
        .level(LogLevel::Info)
        .console(false) // Disable console to focus on logging overhead
        .file("benchmark_async.log")
        .async_logging(50_000) // Large buffer for async logging
        .include_caller(false) // Minimize overhead
        .include_thread(false)
        .datetime_format("%H:%M:%S%.3f")
        .metadata("benchmark", "async")
        .build();

    init(config)?;

    let barrier = Arc::new(Barrier::new(num_threads + 1));
    let mut handles = Vec::new();

    // Start timing threads
    for thread_id in 0..num_threads {
        let barrier_clone = Arc::clone(&barrier);
        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();

            // Log messages
            for i in 0..messages_per_thread {
                if let Err(_) = log_info!("Async benchmark message {} from thread {}", i, thread_id)
                {
                    break; // Stop on error
                }
            }
        });
        handles.push(handle);
    }

    // Start the benchmark
    let start = Instant::now();
    barrier.wait(); // Release all threads

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();

    // Give async logging time to process all messages
    thread::sleep(Duration::from_millis(500));

    Ok(duration)
}
