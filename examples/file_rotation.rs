//! File rotation example for firo_logger.

use firo_logger::{init, log_info, log_warning, LogLevel, LoggerConfig, RotationFrequency};
use std::fs;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== File Rotation Demo ===");
    println!("This demo shows both size-based and time-based log rotation.\n");

    // Create logs directory
    fs::create_dir_all("logs")?;

    // Demo 1: Size-based rotation
    println!("1. Size-based rotation demo:");
    println!("   - Max file size: 1KB");
    println!("   - Keep: 3 backup files");
    println!("   - File: logs/size_rotation.log\n");

    size_based_rotation_demo()?;

    println!("\n{}\n", "=".repeat(50));

    // Demo 2: Time-based rotation
    println!("2. Time-based rotation demo:");
    println!("   - Frequency: Daily");
    println!("   - Keep: 7 backup files");
    println!("   - File: logs/time_rotation.log\n");

    time_based_rotation_demo()?;

    println!("\n{}\n", "=".repeat(50));

    // Show the generated files
    show_generated_files()?;

    println!("\nFile rotation demo completed!");
    println!("Check the 'logs/' directory to see the rotated files.");

    Ok(())
}

fn size_based_rotation_demo() -> Result<(), Box<dyn std::error::Error>> {
    let config = LoggerConfig::builder()
        .level(LogLevel::Info)
        .console(true)
        .file("logs/size_rotation.log")
        .rotate_by_size(1024, 3) // 1KB max, keep 3 backups
        .include_caller(true)
        .datetime_format("%H:%M:%S%.3f")
        .metadata("demo", "size_rotation")
        .build();

    init(config)?;

    log_info!("Starting size-based rotation demo")?;

    // Generate enough log data to trigger rotation
    for i in 1..=20 {
        log_info!(
            "Size rotation test message #{:03} - This is a longer message to help reach the size limit faster and trigger rotation",
            i
        )?;

        if i % 5 == 0 {
            log_warning!("Checkpoint: Generated {} messages so far", i)?;
            thread::sleep(Duration::from_millis(10)); // Small delay
        }
    }

    log_info!("Size-based rotation demo completed")?;

    // Give file system time to flush
    thread::sleep(Duration::from_millis(100));

    Ok(())
}

fn time_based_rotation_demo() -> Result<(), Box<dyn std::error::Error>> {
    let _config = LoggerConfig::builder()
        .level(LogLevel::Info)
        .console(true)
        .file("logs/time_rotation.log")
        .rotate_by_time(RotationFrequency::Daily, 7) // Daily rotation, keep 7 days
        .include_caller(false) // Less verbose for time demo
        .datetime_format("%Y-%m-%d %H:%M:%S")
        .metadata("demo", "time_rotation")
        .build();

    // Note: For demo purposes, we can't easily show time-based rotation
    // as it requires actual time passage. In a real application,
    // this would rotate daily at midnight.

    log_info!("Starting time-based rotation demo")?;
    log_info!("Time-based rotation happens automatically at the configured interval")?;
    log_info!("In this case, it would rotate daily and keep 7 backup files")?;
    log_info!("Files would be named like: time_rotation.log.2025-08-13")?;

    // Simulate some application activity
    for i in 1..=5 {
        log_info!(
            "Time rotation demo message #{} - Normal application logging",
            i
        )?;
        thread::sleep(Duration::from_millis(50));
    }

    log_info!("Time-based rotation demo completed")?;

    // Give file system time to flush
    thread::sleep(Duration::from_millis(100));

    Ok(())
}

fn show_generated_files() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generated files in logs/ directory:");

    let entries = fs::read_dir("logs")?;
    let mut files: Vec<_> = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() {
                Some((
                    path.file_name()?.to_string_lossy().to_string(),
                    fs::metadata(&path).ok()?.len(),
                ))
            } else {
                None
            }
        })
        .collect();

    files.sort_by(|a, b| a.0.cmp(&b.0));

    if files.is_empty() {
        println!("  (No files found)");
    } else {
        for (filename, size) in files {
            println!("  📄 {} ({} bytes)", filename, size);
        }
    }

    // Show content of main log files
    show_file_content("logs/size_rotation.log")?;
    show_file_content("logs/time_rotation.log")?;

    Ok(())
}

fn show_file_content(filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(content) = fs::read_to_string(filepath) {
        println!("\n📋 Content of {}:", filepath);
        let lines: Vec<&str> = content.lines().collect();

        if lines.len() <= 3 {
            // Show all lines if 3 or fewer
            for line in lines {
                println!("  {}", line);
            }
        } else {
            // Show first 2 and last 1 lines with ellipsis
            println!("  {}", lines[0]);
            println!("  {}", lines[1]);
            println!("  ... ({} more lines) ...", lines.len() - 3);
            println!("  {}", lines[lines.len() - 1]);
        }
    }

    Ok(())
}
