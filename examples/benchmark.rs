use std::time::{Duration, Instant};
use std::net::UdpSocket;
use udcn_common::{serialize_interest, serialize_data, hash_name};
use rand;

fn main() -> anyhow::Result<()> {
    println!("µDCN Performance Benchmark");
    println!("==========================");
    
    benchmark_serialization()?;
    benchmark_name_hashing()?;
    benchmark_udp_throughput()?;
    
    Ok(())
}

fn benchmark_serialization() -> anyhow::Result<()> {
    println!("\n1. Packet Serialization Benchmark");
    println!("----------------------------------");
    
    let iterations = 100_000;
    let name = "/benchmark/test/data";
    let content = b"Benchmark data content";
    
    // Interest serialization
    let start = Instant::now();
    for i in 0..iterations {
        let _packet = serialize_interest(name, i);
    }
    let interest_duration = start.elapsed();
    
    // Data serialization  
    let start = Instant::now();
    for i in 0..iterations {
        let _packet = serialize_data(name, content, i);
    }
    let data_duration = start.elapsed();
    
    println!("Interest serialization: {:.2} µs/packet ({:.0} packets/sec)", 
        interest_duration.as_micros() as f64 / iterations as f64,
        iterations as f64 / interest_duration.as_secs_f64());
    
    println!("Data serialization:     {:.2} µs/packet ({:.0} packets/sec)",
        data_duration.as_micros() as f64 / iterations as f64, 
        iterations as f64 / data_duration.as_secs_f64());
    
    Ok(())
}

fn benchmark_name_hashing() -> anyhow::Result<()> {
    println!("\n2. Name Hashing Benchmark");
    println!("-------------------------");
    
    let iterations = 1_000_000;
    let names = vec![
        "/short",
        "/medium/length/name", 
        "/very/long/hierarchical/name/with/many/components/for/testing/performance",
    ];
    
    for name in &names {
        let start = Instant::now();
        for _ in 0..iterations {
            let _hash = hash_name(name.as_bytes());
        }
        let duration = start.elapsed();
        
        println!("{}: {:.2} ns/hash ({:.0} hashes/sec)",
            name,
            duration.as_nanos() as f64 / iterations as f64,
            iterations as f64 / duration.as_secs_f64());
    }
    
    Ok(())
}

fn benchmark_udp_throughput() -> anyhow::Result<()> {
    println!("\n3. UDP Throughput Benchmark"); 
    println!("----------------------------");
    
    let server_socket = UdpSocket::bind("127.0.0.1:0")?;
    let server_addr = server_socket.local_addr()?;
    let client_socket = UdpSocket::bind("127.0.0.1:0")?;
    
    let iterations = 10_000;
    let name = "/benchmark/throughput";
    
    // Send interests
    let start = Instant::now();
    for i in 0..iterations {
        let packet = serialize_interest(name, i);
        client_socket.send_to(&packet, server_addr)?;
    }
    let send_duration = start.elapsed();
    
    println!("UDP send throughput:    {:.2} µs/packet ({:.0} packets/sec)",
        send_duration.as_micros() as f64 / iterations as f64,
        iterations as f64 / send_duration.as_secs_f64());
    
    // Receive loop (simplified - in practice would be separate thread)
    server_socket.set_read_timeout(Some(Duration::from_millis(10)))?;
    let mut received = 0;
    let mut buf = [0u8; 1024];
    
    let start = Instant::now();
    while received < iterations && start.elapsed() < Duration::from_secs(5) {
        if let Ok(_) = server_socket.recv_from(&mut buf) {
            received += 1;
        }
    }
    let recv_duration = start.elapsed();
    
    if received > 0 {
        println!("UDP recv throughput:    {:.2} µs/packet ({:.0} packets/sec)",
            recv_duration.as_micros() as f64 / received as f64,
            received as f64 / recv_duration.as_secs_f64());
    }
    
    println!("Received {} out of {} packets", received, iterations);
    
    Ok(())
}