/// RMIR Binary Decoder
///
/// Decodes RMIR bytecode back to RmirProgram (symmetric with encoder)

use std::io;
use blake3::Hasher;

/// Decodes RMIR bytecode format
pub struct RmirDecoder<'a> {
    bytes: &'a [u8],
    offset: usize,
}

#[derive(Clone, Debug)]
pub struct RmirProgram {
    pub instructions: Vec<(u8, Vec<u32>)>, // (opcode, args)
    pub proof_goals: Vec<ProofGoal>,
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

impl<'a> RmirDecoder<'a> {
    /// Create decoder from bytecode
    pub fn new(bytes: &'a [u8]) -> Result<Self, io::Error> {
        if bytes.len() < 16 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Bytecode too short"));
        }

        // Verify magic
        if &bytes[0..4] != b"RMIR" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid magic"));
        }

        // Verify version
        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        if version != 0x00000001 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Unsupported version"));
        }

        Ok(Self { bytes, offset: 0 })
    }

    /// Verify checksum
    pub fn verify_checksum(&self) -> Result<(), io::Error> {
        if self.bytes.len() < 32 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "No checksum"));
        }

        // Compute hash of all data except checksum
        let data_len = self.bytes.len() - 32;
        let mut hasher = Hasher::new();
        hasher.update(&self.bytes[..data_len]);
        let computed_hash = hasher.finalize();

        // Compare with stored checksum
        let stored_checksum = &self.bytes[data_len..];
        if computed_hash.as_bytes() != stored_checksum {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Checksum mismatch"));
        }

        Ok(())
    }

    /// Decode entire program
    pub fn decode_all(mut self) -> Result<RmirProgram, io::Error> {
        // Skip header (16 bytes)
        self.offset = 16;

        // Skip metadata
        let meta_len = self.read_u32_le()? as usize;
        self.offset += meta_len;

        // Read instructions
        let instr_count = self.read_u32_le()? as usize;
        let mut instructions = Vec::with_capacity(instr_count);

        for _ in 0..instr_count {
            let opcode = self.read_u8()?;
            let arg_count = self.read_u8()? as usize;
            let mut args = Vec::with_capacity(arg_count);

            for _ in 0..arg_count {
                args.push(self.read_varint()? as u32);
            }

            instructions.push((opcode, args));
        }

        // Read proof goals
        let goal_count = self.read_u32_le()? as usize;
        let mut proof_goals = Vec::with_capacity(goal_count);

        for _ in 0..goal_count {
            let goal_type = self.read_u8()?;
            let goal = match goal_type {
                0x00 => ProofGoal::OwnershipInvariant(self.read_u32_le()?),
                0x01 => ProofGoal::RegionSafety(self.read_u32_le()?),
                0x02 => ProofGoal::LinearityConstraint(self.read_u32_le()?),
                0x03 => ProofGoal::EffectOrdering(self.read_u32_le()?, self.read_u32_le()?),
                0x04 => ProofGoal::BoundsCheck(self.read_u32_le()?, self.read_u32_le()?),
                0x05 => ProofGoal::EffectPrecondition(self.read_u8()?),
                _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown proof goal")),
            };
            proof_goals.push(goal);
        }

        Ok(RmirProgram {
            instructions,
            proof_goals,
        })
    }

    /// Read little-endian u8
    fn read_u8(&mut self) -> Result<u8, io::Error> {
        if self.offset >= self.bytes.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let value = self.bytes[self.offset];
        self.offset += 1;
        Ok(value)
    }

    /// Read little-endian u32
    fn read_u32_le(&mut self) -> Result<u32, io::Error> {
        if self.offset + 4 > self.bytes.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let bytes = &self.bytes[self.offset..self.offset + 4];
        self.offset += 4;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read variable-length integer
    fn read_varint(&mut self) -> Result<u64, io::Error> {
        let b1 = self.read_u8()? as u64;

        if b1 <= 0x7F {
            Ok(b1)
        } else if b1 >= 0x80 && b1 <= 0xBF {
            let b2 = self.read_u8()? as u64;
            Ok(((b1 & 0x3F) << 8) | b2)
        } else if b1 == 0xC0 {
            let val = self.read_u32_le()? as u64;
            Ok(val)
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid varint"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rmir::encoder::RmirEncoder;

    #[test]
    fn test_decode_header() {
        let bytes = b"RMIR\x01\x00\x00\x00\x00\x00\x00\x00";
        let decoder = RmirDecoder::new(bytes).unwrap();
        assert_eq!(decoder.offset, 0);
    }

    #[test]
    fn test_decode_invalid_magic() {
        let bytes = b"XXXX\x01\x00\x00\x00\x00\x00\x00\x00";
        assert!(RmirDecoder::new(bytes).is_err());
    }

    #[test]
    fn test_decode_invalid_version() {
        let bytes = b"RMIR\x02\x00\x00\x00\x00\x00\x00\x00";
        assert!(RmirDecoder::new(bytes).is_err());
    }

    #[test]
    fn test_roundtrip_simple() {
        let mut encoder = RmirEncoder::new();
        encoder.encode_header().unwrap();
        encoder.encode_metadata(1234567890, "test.rs", "0.1.0").unwrap();
        encoder.encode_instruction_header(1).unwrap();
        encoder.encode_instruction(0x09, &[1]).unwrap();
        encoder.encode_proof_goals().unwrap();

        let bytecode = encoder.finish().unwrap();
        let decoder = RmirDecoder::new(&bytecode).unwrap();

        decoder.verify_checksum().unwrap();
        let program = decoder.decode_all().unwrap();

        assert_eq!(program.instructions.len(), 1);
        assert_eq!(program.instructions[0].0, 0x09);
        assert_eq!(program.instructions[0].1, vec![1]);
    }

    #[test]
    fn test_roundtrip_multiple_instructions() {
        let mut encoder = RmirEncoder::new();
        encoder.encode_header().unwrap();
        encoder.encode_metadata(0, "", "").unwrap();
        encoder.encode_instruction_header(3).unwrap();
        encoder.encode_instruction(0x09, &[1]).unwrap();
        encoder.encode_instruction(0x11, &[1, 2, 3, 4]).unwrap();
        encoder.encode_instruction(0x0A, &[1]).unwrap();
        encoder.encode_proof_goals().unwrap();

        let bytecode = encoder.finish().unwrap();
        let decoder = RmirDecoder::new(&bytecode).unwrap();
        decoder.verify_checksum().unwrap();
        let program = decoder.decode_all().unwrap();

        assert_eq!(program.instructions.len(), 3);
    }

    #[test]
    fn test_roundtrip_with_proof_goals() {
        let mut encoder = RmirEncoder::new();
        encoder.encode_header().unwrap();
        encoder.encode_metadata(0, "", "").unwrap();
        encoder.encode_instruction_header(1).unwrap();
        encoder.encode_instruction(0x09, &[1]).unwrap();

        encoder.add_proof_goal(ProofGoal::RegionSafety(1));
        encoder.add_proof_goal(ProofGoal::OwnershipInvariant(2));

        encoder.encode_proof_goals().unwrap();

        let bytecode = encoder.finish().unwrap();
        let decoder = RmirDecoder::new(&bytecode).unwrap();
        decoder.verify_checksum().unwrap();
        let program = decoder.decode_all().unwrap();

        assert_eq!(program.proof_goals.len(), 2);
    }

    #[test]
    fn test_checksum_detection() {
        let mut encoder = RmirEncoder::new();
        encoder.encode_header().unwrap();
        encoder.encode_metadata(0, "", "").unwrap();
        encoder.encode_instruction_header(0).unwrap();
        encoder.encode_proof_goals().unwrap();

        let mut bytecode = encoder.finish().unwrap();

        // Corrupt last byte of checksum
        bytecode[bytecode.len() - 1] ^= 0xFF;

        let decoder = RmirDecoder::new(&bytecode).unwrap();
        assert!(decoder.verify_checksum().is_err());
    }
}
