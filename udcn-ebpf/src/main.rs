#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::xdp_action, 
    macros::{xdp, map},
    maps::{HashMap, LruHashMap, Array},
    programs::XdpContext,
};
use udcn_common::{PitEntry, CacheEntry, PacketStats};

#[map]
static PIT: HashMap<u32, PitEntry> = HashMap::with_max_entries(1024, 0);

#[map]
static CONTENT_STORE: LruHashMap<u32, CacheEntry> = LruHashMap::with_max_entries(512, 0);

#[map]
static STATS: Array<PacketStats> = Array::with_max_entries(1, 0);

#[map]
static DATA_CACHE: HashMap<u32, [u8; 256]> = HashMap::with_max_entries(512, 0);

#[xdp]
pub fn udcn(ctx: XdpContext) -> u32 {
    match try_udcn(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

fn try_udcn(ctx: XdpContext) -> Result<u32, u32> {
    let data = ctx.data();
    let data_end = ctx.data_end();
    
    // Count all packets that reach XDP (use drops as a general packet counter)
    update_stats(|stats| stats.drops += 1);
    
    // Ensure we have at least Ethernet (14) + minimal IP (20) bytes
    if data + 34 > data_end {
        return Ok(xdp_action::XDP_PASS);
    }
    

    // Check if this is an IPv4 packet (ethertype = 0x0800)
    let eth_type = unsafe {
        let ptr = (data + 12) as *const u16;
        u16::from_be(*ptr)
    };
    
    
    if eth_type != 0x0800 {
        return Ok(xdp_action::XDP_PASS);
    }

    // Get IP header length (IHL field * 4)
    let ip_ihl = unsafe { *((data + 14) as *const u8) } & 0x0f;
    let ip_header_len = (ip_ihl * 4) as usize;
    
    // Ensure we have enough space for IP header + UDP header
    if data + 14 + ip_header_len + 8 > data_end {
        return Ok(xdp_action::XDP_PASS);
    }

    // Check if this is a UDP packet (protocol = 17)
    let ip_protocol = unsafe { *((data + 14 + 9) as *const u8) };
    
    
    if ip_protocol != 17 {
        return Ok(xdp_action::XDP_PASS);
    }

    // Check if this is destined for NDN port 6363
    let udp_header_start = data + 14 + ip_header_len;
    let udp_dst_port = unsafe {
        let ptr = (udp_header_start + 2) as *const u16;
        u16::from_be(*ptr)
    };
    
    // Also check source port for return traffic
    let udp_src_port = unsafe {
        let ptr = (udp_header_start) as *const u16;
        u16::from_be(*ptr)
    };
    
    // Count UDP packets that reach port check
    update_stats(|stats| stats.forwards += 1);
    
    // Check if either source or destination port is 6363 (NDN traffic)
    if udp_dst_port != 6363 && udp_src_port != 6363 {
        return Ok(xdp_action::XDP_PASS);
    }
    
    // Found NDN traffic on port 6363

    // Get UDP payload (NDN packet) start
    let udp_payload_start = udp_header_start + 8;
    
    // Ensure we have at least 2 bytes for NDN header
    if udp_payload_start + 2 > data_end {
        return Ok(xdp_action::XDP_PASS);
    }

    // Get NDN packet type from UDP payload
    let packet_type = unsafe { *(udp_payload_start as *const u8) };
    
    // Quick check: is this potentially an NDN packet?
    if packet_type != 0x05 && packet_type != 0x06 {
        return Ok(xdp_action::XDP_PASS);
    }

    // Count NDN packet types
    update_stats(|stats| {
        if packet_type == 0x05 {
            stats.interest_received += 1;
        } else if packet_type == 0x06 {
            stats.data_received += 1;
        }
    });

    // For Interest packets, we need at least 12 bytes (header + name_hash + nonce)
    if packet_type == 0x05 {
        if udp_payload_start + 12 > data_end {
            return Ok(xdp_action::XDP_PASS);
        }
        
        // Parse Interest packet manually with verified bounds
        let name_hash = unsafe {
            let ptr = (udp_payload_start + 2) as *const u32;
            *ptr
        };
        let nonce = unsafe {
            let ptr = (udp_payload_start + 6) as *const u32;
            *ptr
        };
        
        let interest = udcn_common::InterestPacket::new(name_hash, nonce);
        return handle_interest(interest);
    }
    
    // For Data packets, we need at least 10 bytes (header + name_hash + content_size + signature)
    if packet_type == 0x06 {
        if udp_payload_start + 10 > data_end {
            return Ok(xdp_action::XDP_PASS);
        }
        
        // Parse Data packet manually with verified bounds
        let name_hash = unsafe {
            let ptr = (udp_payload_start + 2) as *const u32;
            *ptr
        };
        let content_size = unsafe {
            let ptr = (udp_payload_start + 6) as *const u16;
            *ptr
        };
        let signature = unsafe {
            let ptr = (udp_payload_start + 8) as *const u32;
            *ptr
        };
        
        let data_pkt = udcn_common::DataPacket::new(name_hash, content_size, signature);
        
        // Create a minimal payload slice for caching
        let payload_len = (data_end - udp_payload_start) as usize;
        let payload = unsafe {
            core::slice::from_raw_parts(udp_payload_start as *const u8, payload_len)
        };
        
        return handle_data(data_pkt, payload);
    }

    Ok(xdp_action::XDP_PASS)
}

fn handle_interest(interest: udcn_common::InterestPacket) -> Result<u32, u32> {
    let name_hash = interest.name_hash;
    
    if let Some(_cache_entry) = unsafe { CONTENT_STORE.get(&name_hash) } {
        update_stats(|stats| stats.cache_hits += 1);
        
        if let Some(_cached_data) = unsafe { DATA_CACHE.get(&name_hash) } {
            return Ok(xdp_action::XDP_TX);
        }
    }

    // Cache miss - will add to PIT

    let pit_entry = PitEntry {
        name_hash,
        face_id: 1,
        timestamp: 0,
    };

    if let Err(_) = unsafe { PIT.insert(&name_hash, &pit_entry, 0) } {
        update_stats(|stats| stats.drops += 1);
        return Ok(xdp_action::XDP_DROP);
    }

    Ok(xdp_action::XDP_PASS)
}

fn handle_data(data_pkt: udcn_common::DataPacket, _full_packet: &[u8]) -> Result<u32, u32> {
    let name_hash = data_pkt.name_hash;
    
    if let Some(_pit_entry) = unsafe { PIT.get(&name_hash) } {
        update_stats(|stats| stats.pit_hits += 1);
        
        let _ = unsafe { PIT.remove(&name_hash) };

        let cache_entry = CacheEntry {
            name_hash,
            data_size: data_pkt.content_size,
            timestamp: 0,
        };

        let _ = unsafe { CONTENT_STORE.insert(&name_hash, &cache_entry, 0) };

        // For now, skip actual data caching to avoid verifier issues
        // In a real implementation, we'd copy packet data here
        
        return Ok(xdp_action::XDP_PASS);
    }

    update_stats(|stats| stats.drops += 1);
    Ok(xdp_action::XDP_DROP)
}

fn update_stats<F>(f: F) 
where 
    F: FnOnce(&mut PacketStats),
{
    if let Some(stats) = STATS.get_ptr_mut(0) {
        unsafe {
            f(&mut *stats);
        }
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[link_section = "license"]
#[no_mangle]
static LICENSE: [u8; 13] = *b"Dual MIT/GPL\0";
