use std::error::Error;

use common::{Encode, Proto};

use serialization::block::BlockDB as ProtoBlockDB;

use protobuf::Message as ProtoMessage;

#[derive(Clone)]
pub struct Meta {
    pub height: u32,
    pub t_ema: f64,
    pub p_ema: f64,
    pub next_difficulty: f64,
    pub total_work: f64,
    pub file_number: Option<u32>,
    pub offset: Option<u32>,
    pub length: Option<u32>,
}

impl Meta {
    pub fn new(height: u32, t_ema: f64, p_ema: f64, next_difficulty: f64, total_work: f64, file_number: Option<u32>, offset: Option<u32>, length: Option<u32>) -> Meta {
        Meta {
            height,
            t_ema,
            p_ema,
            next_difficulty,
            total_work,
            file_number,
            offset,
            length
        }
    }
}

impl Proto for Meta {
    type ProtoType = ProtoBlockDB;
    fn to_proto(&self) -> Result<Self::ProtoType, Box<Error>> {
        let mut proto_meta = Self::ProtoType::new();
        proto_meta.set_height(self.height);
        proto_meta.set_tEMA(self.t_ema);
        proto_meta.set_pEMA(self.p_ema);
        proto_meta.set_nextDifficulty(self.next_difficulty);
        proto_meta.set_totalWork(self.total_work);
        match self.file_number {
            Some(fd) => proto_meta.set_fileNumber(fd),
            None => {}
        }
        match self.offset {
            Some(offset) => proto_meta.set_offset(offset),
            None => {}
        }
        match self.length {
            Some(length) => proto_meta.set_length(length),
            None => {}
        }
        Ok(proto_meta)
    }
}

impl Encode for Meta {
    fn encode(&self) -> Result<Vec<u8>, Box<Error>> {
        let proto_meta = self.to_proto()?;
        Ok(proto_meta.write_to_bytes()?)
    }
}

mod tests {
    use super::*;

    #[test]
    fn it_makes_meta_without_file_info() {
        let height = 150000;
        let t_ema = 30.00;
        let p_ema = 0.000001;
        let next_difficulty = 0.0001;
        let total_work = 1e15;
        let meta = Meta::new(height, t_ema, p_ema, next_difficulty, total_work, None, None, None);

        assert_eq!(meta.height, height);
        assert_eq!(meta.t_ema, t_ema);
        assert_eq!(meta.p_ema, p_ema);
        assert_eq!(meta.total_work, total_work);
        assert_eq!(meta.file_number, None);
        assert_eq!(meta.offset, None);
        assert_eq!(meta.length, None);
    }

    #[test]
    fn it_makes_meta_with_file_info() {
        let height = 123456789;
        let t_ema = 1234.0;
        let p_ema = 0.1234;
        let next_difficulty = 0.012345;
        let total_work = 1e23;
        let offset = 123;
        let file_number = 234;
        let length = 345;
        let meta = Meta::new(height, t_ema, p_ema, next_difficulty, total_work, Some(file_number), Some(offset), Some(length));

        assert_eq!(meta.height, height);
        assert_eq!(meta.t_ema, t_ema);
        assert_eq!(meta.p_ema, p_ema);
        assert_eq!(meta.next_difficulty, next_difficulty);
        assert_eq!(meta.total_work, total_work);
        assert_eq!(meta.offset.unwrap(), offset);
        assert_eq!(meta.file_number.unwrap(), file_number);
        assert_eq!(meta.length.unwrap(), length);
    }
}