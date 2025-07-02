use anyhow::Context as _;
use aya::{
    programs::{Xdp, XdpFlags},
    maps::Array,
};
use clap::{Parser, Subcommand};
#[rustfmt::skip]
use log::{debug, warn, info};
use tokio::{signal, time::{sleep, Duration}};
use std::net::{UdpSocket, SocketAddr};
use udcn_common::{PacketStats, serialize_interest, serialize_data, hash_name};
use rand;

#[derive(Debug, Parser)]
#[command(name = "udcn")]
#[command(about = "A minimal µDCN implementation using eBPF/XDP")]
struct Opt {
    #[clap(short, long, default_value = "udcn0")]
    iface: String,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Run {
        #[clap(long)]
        stats_interval: Option<u64>,
    },
    Send {
        #[clap(short, long)]
        name: String,
        #[clap(short, long, default_value = "127.0.0.1:6363")]
        target: String,
    },
    Serve {
        #[clap(short, long)]
        name: String,
        #[clap(short, long)]
        content: String,
        #[clap(short, long, default_value = "127.0.0.1:6363")]
        bind: String,
    },
    Stats,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    env_logger::init();

    match opt.command {
        Commands::Run { stats_interval } => {
            run_daemon(opt.iface, stats_interval).await
        }
        Commands::Send { name, target } => {
            send_interest(name, target).await
        }
        Commands::Serve { name, content, bind } => {
            serve_data(name, content, bind).await
        }
        Commands::Stats => {
            show_stats().await
        }
    }
}

async fn run_daemon(iface: String, stats_interval: Option<u64>) -> anyhow::Result<()> {
    bump_memlock_rlimit()?;
    
    let mut ebpf = aya::Ebpf::load(aya::include_bytes_aligned!(concat!(
        env!("OUT_DIR"),
        "/udcn"
    )))?;
    
    if let Err(e) = aya_log::EbpfLogger::init(&mut ebpf) {
        warn!("failed to initialize eBPF logger: {e}");
    }
    
    let program: &mut Xdp = ebpf.program_mut("udcn").unwrap().try_into()?;
    program.load()?;
    program.attach(&iface, XdpFlags::default())
        .context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?;

    info!("µDCN XDP program loaded and attached to {}", iface);

    if let Some(interval) = stats_interval {
        let stats_map: Array<_, PacketStats> = Array::try_from(ebpf.take_map("STATS").unwrap())?;
        
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(interval)).await;
                if let Ok(stats) = stats_map.get(&0, 0) {
                    print_stats(&stats);
                }
            }
        });
    }

    let ctrl_c = signal::ctrl_c();
    info!("µDCN daemon running. Press Ctrl-C to exit...");
    ctrl_c.await?;
    info!("Shutting down µDCN daemon...");

    Ok(())
}

async fn send_interest(name: String, target: String) -> anyhow::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let target_addr: SocketAddr = target.parse()?;
    
    let nonce = rand::random::<u32>();
    let interest_packet = serialize_interest(&name, nonce);
    
    socket.send_to(&interest_packet, target_addr)?;
    info!("Sent Interest for '{}' to {}", name, target);
    
    let mut buf = [0u8; 1024];
    match socket.recv_from(&mut buf) {
        Ok((len, addr)) => {
            info!("Received Data response ({} bytes) from {}", len, addr);
        }
        Err(e) => {
            warn!("Failed to receive Data response: {}", e);
        }
    }
    
    Ok(())
}

async fn serve_data(name: String, content: String, bind: String) -> anyhow::Result<()> {
    let socket = UdpSocket::bind(&bind)?;
    info!("Serving content for '{}' on {}", name, bind);
    
    let mut buf = [0u8; 1024];
    
    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, addr)) => {
                if let Some(interest) = udcn_common::parse_interest_packet(&buf[..len]) {
                    let expected_hash = hash_name(name.as_bytes());
                    if interest.name_hash == expected_hash {
                        let signature = rand::random::<u32>();
                        let data_packet = serialize_data(&name, content.as_bytes(), signature);
                        
                        if let Err(e) = socket.send_to(&data_packet, addr) {
                            warn!("Failed to send Data response: {}", e);
                        } else {
                            info!("Sent Data response for '{}' to {}", name, addr);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to receive packet: {}", e);
            }
        }
    }
}

async fn show_stats() -> anyhow::Result<()> {
    bump_memlock_rlimit()?;
    
    let mut ebpf = aya::Ebpf::load(aya::include_bytes_aligned!(concat!(
        env!("OUT_DIR"),
        "/udcn"
    )))?;
    
    let stats_map: Array<_, PacketStats> = Array::try_from(ebpf.take_map("STATS").unwrap())?;
    
    if let Ok(stats) = stats_map.get(&0, 0) {
        print_stats(&stats);
    } else {
        println!("No statistics available");
    }
    
    Ok(())
}

fn print_stats(stats: &PacketStats) {
    println!("µDCN Statistics:");
    println!("================");
    println!("Interest packets received: {}", stats.interest_received);
    println!("Data packets received:     {}", stats.data_received);
    println!("Cache hits:                {}", stats.cache_hits);
    println!("Cache misses:              {}", stats.cache_misses);
    println!("PIT hits:                  {}", stats.pit_hits);
    println!("Forwards:                  {}", stats.forwards);
    println!("Drops:                     {}", stats.drops);
    
    let total_interests = stats.cache_hits + stats.cache_misses;
    if total_interests > 0 {
        let hit_ratio = (stats.cache_hits as f64 / total_interests as f64) * 100.0;
        println!("Cache hit ratio:           {:.2}%", hit_ratio);
    }
}

fn bump_memlock_rlimit() -> anyhow::Result<()> {
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        debug!("remove limit on locked memory failed, ret is: {ret}");
    }
    Ok(())
}
