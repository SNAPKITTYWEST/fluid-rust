/// RMIR Binary Encoder
///
/// Encodes RmirProgram to bytecode format (see spec/RMIR_FORMAT.md)

use std::io::{self, Write};
use blake3::Hasher;

/// Encodes an RMIR program to binary bytecode
pub struct RmirEncoder {
    buffer: Vec<u8>,
    proof_goals: Vec<ProofGoal>,
}

#[derive(Clone, Debug)]
pub enum ProofGoal {
    OwnershipInvariant(u32),
    RegionSafety(u32),
    LinearityConstraint(u32),
    EffectOrdering(u32, u32),
    BoundsCheck(u32, u32),
    EffectPrecondition(u8),
}

impl RmirEncoder {
    /// Create new encoder with metadata
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(4096),
            proof_goals: Vec::new(),
        }
    }

    /// Encode header (magic + version)
    pub fn encode_header(&mut self) -> io::Result<()> {
        // Magic: "RMIR"
        self.buffer.extend_from_slice(b"RMIR");

        // Version: 1
        self.write_u32_le(0x00000001)?;

        // Flags + padding
        self.buffer.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        Ok(())
    }

    /// Encode metadata section
    pub fn encode_metadata(
        &mut self,
        timestamp: u64,
        source_file: &str,
        compiler_version: &str,
    ) -> io::Result<()> {
        // Metadata length (placeholder, fill later)
        let len_pos = self.buffer.len();
        self.write_u32_le(0)?; // Will update

        // Timestamp
        self.write_u64_le(timestamp)?;

        // Source file
        self.write_u32_le(source_file.len() as u32)?;
        self.buffer.extend_from_slice(source_file.as_bytes());

        // Compiler version
        self.buffer.push(compiler_version.len() as u8);
        self.buffer.extend_from_slice(compiler_version.as_bytes());

        // Padding to 16-byte boundary
        let metadata_len = self.buffer.len() - len_pos - 4;
        let padding = (16 - (metadata_len % 16)) % 16;
        for _ in 0..padding {
            self.buffer.push(0x00);
        }

        // Update length field
        let len_bytes = (self.buffer.len() - len_pos - 4) as u32;
        self.buffer[len_pos..len_pos + 4].copy_from_slice(&len_bytes.to_le_bytes());

        Ok(())
    }

    /// Encode instruction section header
    pub fn encode_instruction_header(&mut self, count: u32) -> io::Result<()> {
        self.write_u32_le(count)
    }

    /// Encode a single instruction
    pub fn encode_instruction(&mut self, opcode: u8, args: &[u32]) -> io::Result<()> {
        self.buffer.push(opcode);
        self.buffer.push(args.len() as u8);
        for &arg in args {
            self.write_varint(arg as u64)?;
        }
        Ok(())
    }

    /// Add a proof goal and return its ID
    pub fn add_proof_goal(&mut self, goal: ProofGoal) -> u32 {
        let id = self.proof_goals.len() as u32;
        self.proof_goals.push(goal);
        id
    }

    /// Encode proof goals section
    pub fn encode_proof_goals(&mut self) -> io::Result<()> {
        self.write_u32_le(self.proof_goals.len() as u32)?;

        for goal in &self.proof_goals {
            match goal {
                ProofGoal::OwnershipInvariant(val_id) => {
                    self.buffer.push(0x00);
                    self.write_u32_le(*val_id)?;
                }
                ProofGoal::RegionSafety(region_id) => {
                    self.buffer.push(0x01);
                    self.write_u32_le(*region_id)?;
                }
                ProofGoal::LinearityConstraint(val_id) => {
                    self.buffer.push(0x02);
                    self.write_u32_le(*val_id)?;
                }
                ProofGoal::EffectOrdering(e1, e2) => {
                    self.buffer.push(0x03);
                    self.write_u32_le(*e1)?;
                    self.write_u32_le(*e2)?;
                }
                ProofGoal::BoundsCheck(idx, len) => {
                    self.buffer.push(0x04);
                    self.write_u32_le(*idx)?;
                    self.write_u32_le(*len)?;
                }
                ProofGoal::EffectPrecondition(kind) => {
                    self.buffer.push(0x05);
                    self.buffer.push(*kind);
                }
            }
        }

        Ok(())
    }

    /// Finish encoding and append checksum
    pub fn finish(mut self) -> io::Result<Vec<u8>> {
        // Compute Blake3 checksum of all data before checksum field
        let mut hasher = Hasher::new();
        hasher.update(&self.buffer);
        let hash = hasher.finalize();

        // Append checksum
        self.buffer.extend_from_slice(hash.as_bytes());

        Ok(self.buffer)
    }

    /// Write little-endian u32
    fn write_u32_le(&mut self, value: u32) -> io::Result<()> {
        self.buffer.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// Write little-endian u64
    fn write_u64_le(&mut self, value: u64) -> io::Result<()> {
        self.buffer.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// Write variable-length integer
    /// 0-127: 1 byte
    /// 128-16383: 2 bytes
    /// 16384+: 5 bytes
    fn write_varint(&mut self, value: u64) -> io::Result<()> {
        if value <= 0x7F {
            self.buffer.push(value as u8);
        } else if value <= 0x3FFF {
            let b1 = (0x80 | ((value >> 8) as u8)) as u8;
            let b2 = (value & 0xFF) as u8;
            self.buffer.push(b1);
            self.buffer.push(b2);
        } else {
            self.buffer.push(0xC0);
            self.buffer.extend_from_slice(&(value as u32).to_le_bytes());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_header() {
        let mut encoder = RmirEncoder::new();
        encoder.encode_header().unwrap();
        assert!(encoder.buffer.starts_with(b"RMIR"));
        assert_eq!(encoder.buffer[4..8], [0x01, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_encode_instruction() {
        let mut encoder = RmirEncoder::new();
        encoder.encode_instruction(0x09, &[1]).unwrap(); // RegionEnter(1)
        assert_eq!(encoder.buffer[0], 0x09); // opcode
        assert_eq!(encoder.buffer[1], 1); // arg count
    }

    #[test]
    fn test_proof_goal_tracking() {
        let mut encoder = RmirEncoder::new();
        let id1 = encoder.add_proof_goal(ProofGoal::RegionSafety(1));
        let id2 = encoder.add_proof_goal(ProofGoal::OwnershipInvariant(2));

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(encoder.proof_goals.len(), 2);
    }

    #[test]
    fn test_varint_encoding() {
        let mut encoder = RmirEncoder::new();

        // Small value: 1 byte
        encoder.write_varint(42).unwrap();
        assert_eq!(encoder.buffer[0], 42);

        // Medium value: 2 bytes
        encoder.buffer.clear();
        encoder.write_varint(256).unwrap();
        assert_eq!(encoder.buffer[0] & 0x80, 0x80); // high bit set

        // Large value: 5 bytes
        encoder.buffer.clear();
        encoder.write_varint(100000).unwrap();
        assert_eq!(encoder.buffer[0], 0xC0); // marker
    }

    #[test]
    fn test_finish_adds_checksum() {
        let mut encoder = RmirEncoder::new();
        encoder.buffer.extend_from_slice(b"TEST");
        let result = encoder.finish().unwrap();

        // Should have 4 bytes data + 32 bytes checksum
        assert_eq!(result.len(), 4 + 32);
    }
}
