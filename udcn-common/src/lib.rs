#![no_std]

use core::mem;

pub const NDN_ETHERTYPE: u16 = 0x8624;
pub const NDN_UDP_PORT: u16 = 6363;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TlvType {
    Interest = 0x05,
    Data = 0x06,
    Name = 0x07,
    NameComponent = 0x08,
    Nonce = 0x0A,
    Content = 0x15,
    MetaInfo = 0x14,
    SignatureInfo = 0x16,
    SignatureValue = 0x17,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TlvHeader {
    pub tlv_type: u8,
    pub length: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NdnPacketHeader {
    pub packet_type: u8,
    pub packet_length: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct InterestPacket {
    pub header: NdnPacketHeader,
    pub name_hash: u32,
    pub nonce: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DataPacket {
    pub header: NdnPacketHeader,
    pub name_hash: u32,
    pub content_size: u16,
    pub signature: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PitEntry {
    pub name_hash: u32,
    pub face_id: u32,
    pub timestamp: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CacheEntry {
    pub name_hash: u32,
    pub data_size: u16,
    pub timestamp: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PacketStats {
    pub interest_received: u32,
    pub data_received: u32,
    pub cache_hits: u32,
    pub cache_misses: u32,
    pub pit_hits: u32,
    pub forwards: u32,
    pub drops: u32,
}

// Implement Pod trait for Aya - PacketStats is just u32 fields so it's safe
#[cfg(feature = "user")]
unsafe impl aya::Pod for PacketStats {}

pub fn hash_name(name: &[u8]) -> u32 {
    const FNV_OFFSET_BASIS: u32 = 0x811c9dc5;
    const FNV_PRIME: u32 = 0x01000193;
    
    let mut hash = FNV_OFFSET_BASIS;
    for byte in name {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

impl TlvHeader {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 2 {
            return None;
        }
        Some(Self {
            tlv_type: data[0],
            length: data[1],
        })
    }
}

impl InterestPacket {
    pub fn new(name_hash: u32, nonce: u32) -> Self {
        Self {
            header: NdnPacketHeader {
                packet_type: TlvType::Interest as u8,
                packet_length: mem::size_of::<InterestPacket>() as u8,
            },
            name_hash,
            nonce,
        }
    }
}

impl DataPacket {
    pub fn new(name_hash: u32, content_size: u16, signature: u32) -> Self {
        Self {
            header: NdnPacketHeader {
                packet_type: TlvType::Data as u8,
                packet_length: mem::size_of::<DataPacket>() as u8,
            },
            name_hash,
            content_size,
            signature,
        }
    }
}

pub fn parse_interest_packet(data: &[u8]) -> Option<InterestPacket> {
    if data.len() < mem::size_of::<InterestPacket>() {
        return None;
    }
    
    let packet = unsafe { &*(data.as_ptr() as *const InterestPacket) };
    
    if packet.header.packet_type == TlvType::Interest as u8 {
        Some(*packet)
    } else {
        None
    }
}

pub fn parse_data_packet(data: &[u8]) -> Option<DataPacket> {
    if data.len() < mem::size_of::<DataPacket>() {
        return None;
    }
    
    let packet = unsafe { &*(data.as_ptr() as *const DataPacket) };
    
    if packet.header.packet_type == TlvType::Data as u8 {
        Some(*packet)
    } else {
        None
    }
}

pub fn is_ndn_packet(data: &[u8]) -> bool {
    if data.len() < mem::size_of::<NdnPacketHeader>() {
        return false;
    }
    
    let header = unsafe { &*(data.as_ptr() as *const NdnPacketHeader) };
    header.packet_type == TlvType::Interest as u8 || header.packet_type == TlvType::Data as u8
}

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
pub fn serialize_interest(name: &str, nonce: u32) -> std::vec::Vec<u8> {
    let name_hash = hash_name(name.as_bytes());
    let packet = InterestPacket::new(name_hash, nonce);
    let bytes = unsafe {
        core::slice::from_raw_parts(
            &packet as *const _ as *const u8,
            mem::size_of::<InterestPacket>(),
        )
    };
    bytes.to_vec()
}

#[cfg(feature = "std")]
pub fn serialize_data(name: &str, content: &[u8], signature: u32) -> std::vec::Vec<u8> {
    let name_hash = hash_name(name.as_bytes());
    let packet = DataPacket::new(name_hash, content.len() as u16, signature);
    let mut result = std::vec::Vec::new();
    
    let packet_bytes = unsafe {
        core::slice::from_raw_parts(
            &packet as *const _ as *const u8,
            mem::size_of::<DataPacket>(),
        )
    };
    
    result.extend_from_slice(packet_bytes);
    result.extend_from_slice(content);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interest_packet_creation() {
        let name_hash = 0x12345678;
        let nonce = 0x9ABCDEF0;
        
        let interest = InterestPacket::new(name_hash, nonce);
        
        assert_eq!(interest.header.packet_type, TlvType::Interest as u8);
        assert_eq!(interest.name_hash, name_hash);
        assert_eq!(interest.nonce, nonce);
        assert_eq!(interest.header.packet_length as usize, core::mem::size_of::<InterestPacket>());
    }

    #[test]
    fn test_data_packet_creation() {
        let name_hash = 0x12345678;
        let content_size = 100;
        let signature = 0x9ABCDEF0;
        
        let data = DataPacket::new(name_hash, content_size, signature);
        
        assert_eq!(data.header.packet_type, TlvType::Data as u8);
        assert_eq!(data.name_hash, name_hash);
        assert_eq!(data.content_size, content_size);
        assert_eq!(data.signature, signature);
        assert_eq!(data.header.packet_length as usize, core::mem::size_of::<DataPacket>());
    }

    #[test]
    fn test_hash_consistency() {
        let name = b"/test/data";
        let hash1 = hash_name(name);
        let hash2 = hash_name(name);
        
        assert_eq!(hash1, hash2, "Hash should be consistent for same input");
    }

    #[test]
    fn test_hash_different_names() {
        let name1 = b"/test/data1";
        let name2 = b"/test/data2";
        let hash1 = hash_name(name1);
        let hash2 = hash_name(name2);
        
        assert_ne!(hash1, hash2, "Different names should have different hashes");
    }

    #[test]
    fn test_packet_structures() {
        // Verify structures are at least the minimum expected size
        assert!(core::mem::size_of::<InterestPacket>() >= 12);
        assert!(core::mem::size_of::<DataPacket>() >= 12);
        assert_eq!(core::mem::size_of::<TlvHeader>(), 2);
        assert_eq!(core::mem::size_of::<NdnPacketHeader>(), 2);
        
        // Test that packet headers are correct type
        let interest = InterestPacket::new(0, 0);
        assert_eq!(interest.header.packet_type, TlvType::Interest as u8);
        
        let data = DataPacket::new(0, 0, 0);
        assert_eq!(data.header.packet_type, TlvType::Data as u8);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_interest_serialization() {
        let name = "/test/data";
        let nonce = 0x12345678;
        
        let serialized = serialize_interest(name, nonce);
        
        // Should start with Interest TLV type
        assert_eq!(serialized[0], TlvType::Interest as u8);
        
        // Should be correct length
        assert_eq!(serialized.len(), core::mem::size_of::<InterestPacket>());
        
        // Should be able to parse back
        let parsed = parse_interest_packet(&serialized).unwrap();
        assert_eq!(parsed.nonce, nonce);
        assert_eq!(parsed.name_hash, hash_name(name.as_bytes()));
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_data_serialization() {
        let name = "/test/data";
        let content = b"Hello, NDN!";
        let signature = 0x9ABCDEF0;
        
        let serialized = serialize_data(name, content, signature);
        
        // Should start with Data TLV type
        assert_eq!(serialized[0], TlvType::Data as u8);
        
        // Should contain the content
        assert!(serialized.len() > core::mem::size_of::<DataPacket>());
        
        // Should be able to parse back the header
        let parsed = parse_data_packet(&serialized).unwrap();
        assert_eq!(parsed.signature, signature);
        assert_eq!(parsed.name_hash, hash_name(name.as_bytes()));
        assert_eq!(parsed.content_size, content.len() as u16);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_is_ndn_packet() {
        let interest = serialize_interest("/test", 123);
        let data = serialize_data("/test", b"content", 456);
        let invalid = std::vec![0xFF, 0x00];
        
        assert!(is_ndn_packet(&interest));
        assert!(is_ndn_packet(&data));
        assert!(!is_ndn_packet(&invalid));
        assert!(!is_ndn_packet(&[]));
    }
}