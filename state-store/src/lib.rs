use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};

const MAGIC: &[u8; 4] = b"AAGF";
const VERSION: u32 = 1;

const TAG_END: u8 = 0xFF;
const TAG_SETPOINT: u8 = 0x01;
const TAG_LEARNING_RATE: u8 = 0x02;
const TAG_ENERGY: u8 = 0x03;
const TAG_REWARD: u8 = 0x04;
const TAG_BIRTH_TICK: u8 = 0x05;
const TAG_LABEL: u8 = 0x06;

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalState {
    pub tick: u64,
    pub global_setpoint: f64,
    pub system_inertia: f64,
    pub alignment_score: f64,
    pub complexity_factor: f64,
    pub accumulated_reward: f64,
    pub success_count: u64,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            tick: 0,
            global_setpoint: 0.5,
            system_inertia: 0.1,
            alignment_score: 0.0,
            complexity_factor: 1.0,
            accumulated_reward: 0.0,
            success_count: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentRecord {
    pub global_setpoint: f64,
    pub learning_rate: f64,
    pub energy: f64,
    pub accumulated_reward: f64,
    pub birth_tick: u32,
    pub label: String,
}

pub struct BinarySerializer;

impl BinarySerializer {
    pub fn save(path: &str, state: &GlobalState, agents: &[AgentRecord]) -> io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(MAGIC)?;
        file.write_all(&VERSION.to_le_bytes())?;
        file.write_all(&state.tick.to_le_bytes())?;
        file.write_all(&state.global_setpoint.to_le_bytes())?;
        file.write_all(&state.system_inertia.to_le_bytes())?;
        file.write_all(&state.alignment_score.to_le_bytes())?;
        file.write_all(&state.complexity_factor.to_le_bytes())?;
        file.write_all(&state.accumulated_reward.to_le_bytes())?;
        file.write_all(&state.success_count.to_le_bytes())?;

        file.write_all(&(agents.len() as u32).to_le_bytes())?;
        for agent in agents {
            Self::write_tlv_f64(&mut file, TAG_SETPOINT, agent.global_setpoint)?;
            Self::write_tlv_f64(&mut file, TAG_LEARNING_RATE, agent.learning_rate)?;
            Self::write_tlv_f64(&mut file, TAG_ENERGY, agent.energy)?;
            Self::write_tlv_f64(&mut file, TAG_REWARD, agent.accumulated_reward)?;
            Self::write_tlv_u32(&mut file, TAG_BIRTH_TICK, agent.birth_tick)?;
            Self::write_tlv_string(&mut file, TAG_LABEL, &agent.label)?;
            file.write_all(&[TAG_END])?;
        }
        Ok(())
    }

    pub fn load(path: &str) -> io::Result<(GlobalState, Vec<AgentRecord>)> {
        let mut file = File::open(path)?;

        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid magic"));
        }

        let mut u32_buf = [0u8; 4];
        file.read_exact(&mut u32_buf)?;
        if u32::from_le_bytes(u32_buf) != VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Version mismatch",
            ));
        }

        let mut u64_buf = [0u8; 8];
        let mut f64_buf = [0u8; 8];

        file.read_exact(&mut u64_buf)?;
        let tick = u64::from_le_bytes(u64_buf);

        file.read_exact(&mut f64_buf)?;
        let global_setpoint = f64::from_le_bytes(f64_buf);

        file.read_exact(&mut f64_buf)?;
        let system_inertia = f64::from_le_bytes(f64_buf);

        file.read_exact(&mut f64_buf)?;
        let alignment_score = f64::from_le_bytes(f64_buf);

        file.read_exact(&mut f64_buf)?;
        let complexity_factor = f64::from_le_bytes(f64_buf);

        file.read_exact(&mut f64_buf)?;
        let accumulated_reward = f64::from_le_bytes(f64_buf);

        file.read_exact(&mut u64_buf)?;
        let success_count = u64::from_le_bytes(u64_buf);

        let state = GlobalState {
            tick,
            global_setpoint,
            system_inertia,
            alignment_score,
            complexity_factor,
            accumulated_reward,
            success_count,
        };

        file.read_exact(&mut u32_buf)?;
        let count = u32::from_le_bytes(u32_buf) as usize;
        let mut agents = Vec::with_capacity(count);

        for _ in 0..count {
            let mut agent = AgentRecord {
                global_setpoint: 0.0,
                learning_rate: 1.0,
                energy: 1.0,
                accumulated_reward: 0.0,
                birth_tick: 0,
                label: String::new(),
            };

            loop {
                let mut tag_buf = [0u8; 1];
                file.read_exact(&mut tag_buf)?;
                let tag = tag_buf[0];
                if tag == TAG_END {
                    break;
                }

                file.read_exact(&mut u32_buf)?;
                let len = u32::from_le_bytes(u32_buf) as usize;

                match tag {
                    TAG_SETPOINT => {
                        file.read_exact(&mut f64_buf)?;
                        agent.global_setpoint = f64::from_le_bytes(f64_buf);
                    }
                    TAG_LEARNING_RATE => {
                        file.read_exact(&mut f64_buf)?;
                        agent.learning_rate = f64::from_le_bytes(f64_buf);
                    }
                    TAG_ENERGY => {
                        file.read_exact(&mut f64_buf)?;
                        agent.energy = f64::from_le_bytes(f64_buf);
                    }
                    TAG_REWARD => {
                        file.read_exact(&mut f64_buf)?;
                        agent.accumulated_reward = f64::from_le_bytes(f64_buf);
                    }
                    TAG_BIRTH_TICK => {
                        file.read_exact(&mut u32_buf)?;
                        agent.birth_tick = u32::from_le_bytes(u32_buf);
                    }
                    TAG_LABEL => {
                        let mut str_buf = vec![0u8; len];
                        file.read_exact(&mut str_buf)?;
                        agent.label = String::from_utf8_lossy(&str_buf).into_owned();
                    }
                    _ => {
                        file.seek(SeekFrom::Current(len as i64))?;
                    }
                }
            }
            agents.push(agent);
        }
        Ok((state, agents))
    }

    fn write_tlv_f64(file: &mut File, tag: u8, val: f64) -> io::Result<()> {
        file.write_all(&[tag])?;
        file.write_all(&8u32.to_le_bytes())?;
        file.write_all(&val.to_le_bytes())?;
        Ok(())
    }

    fn write_tlv_u32(file: &mut File, tag: u8, val: u32) -> io::Result<()> {
        file.write_all(&[tag])?;
        file.write_all(&4u32.to_le_bytes())?;
        file.write_all(&val.to_le_bytes())?;
        Ok(())
    }

    fn write_tlv_string(file: &mut File, tag: u8, val: &str) -> io::Result<()> {
        file.write_all(&[tag])?;
        file.write_all(&(val.len() as u32).to_le_bytes())?;
        file.write_all(val.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_roundtrip() {
        let path = "test_state.bin";
        let state = GlobalState {
            tick: 42,
            global_setpoint: 0.7,
            system_inertia: 0.3,
            alignment_score: 0.85,
            complexity_factor: 1.5,
            accumulated_reward: 2.3,
            success_count: 10,
        };
        let agents = vec![AgentRecord {
            global_setpoint: 0.5,
            learning_rate: 1.2,
            energy: 0.9,
            accumulated_reward: 1.1,
            birth_tick: 100,
            label: "agent_1".to_string(),
        }];

        BinarySerializer::save(path, &state, &agents).unwrap();
        let (loaded_state, loaded_agents) = BinarySerializer::load(path).unwrap();

        assert_eq!(state.tick, loaded_state.tick);
        assert!((state.global_setpoint - loaded_state.global_setpoint).abs() < 1e-9);
        assert_eq!(agents.len(), loaded_agents.len());
        assert_eq!(agents[0].label, loaded_agents[0].label);

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_invalid_magic() {
        let path = "test_bad.bin";
        fs::write(path, b"XXXX").unwrap();
        assert!(BinarySerializer::load(path).is_err());
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_forward_compatibility() {
        let path = "test_forward.bin";
        let mut file = File::create(path).unwrap();
        file.write_all(MAGIC).unwrap();
        file.write_all(&VERSION.to_le_bytes()).unwrap();

        file.write_all(&0u64.to_le_bytes()).unwrap(); 
        file.write_all(&0.5f64.to_le_bytes()).unwrap(); 
        file.write_all(&0.1f64.to_le_bytes()).unwrap(); 
        file.write_all(&0.0f64.to_le_bytes()).unwrap(); 
        file.write_all(&1.0f64.to_le_bytes()).unwrap(); 
        file.write_all(&0.0f64.to_le_bytes()).unwrap(); 
        file.write_all(&0u64.to_le_bytes()).unwrap(); 

        file.write_all(&1u32.to_le_bytes()).unwrap(); 
        file.write_all(&[TAG_SETPOINT]).unwrap();
        file.write_all(&8u32.to_le_bytes()).unwrap();
        file.write_all(&0.5f64.to_le_bytes()).unwrap();

        
        file.write_all(&[0x99]).unwrap(); 
        file.write_all(&4u32.to_le_bytes()).unwrap();
        file.write_all(&[0u8; 4]).unwrap();

        file.write_all(&[TAG_END]).unwrap();

        let (state, agents) = BinarySerializer::load(path).unwrap();
        assert_eq!(state.tick, 0);
        assert_eq!(agents.len(), 1);
        assert!((agents[0].global_setpoint - 0.5).abs() < 1e-9);

        fs::remove_file(path).ok();
    }
}
