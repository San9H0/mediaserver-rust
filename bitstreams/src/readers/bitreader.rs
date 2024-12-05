use crate::readers::errors::BitReaderError;

pub struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        BitReader { data, pos: 0 }
    }

    pub fn as_ref(&self) -> &[u8] {
        self.data
    }

    pub fn read_bits<T>(&mut self, n: usize) -> Result<T, BitReaderError>
    where
        T: TryFrom<u32>,
    {
        if n > 31 {
            return Err(BitReaderError::InvalidParameter(n));
        }
        let mut value = 0;
        let mut bits_read = 0;
        let mut bit_offset = self.pos % 8;
        let mut byte_offset = self.pos / 8;

        while bits_read < n {
            if byte_offset >= self.data.len() {
                return Err(BitReaderError::EndOfStream);
            }
            let remaining_bits_in_byte = 8 - bit_offset;
            let bits_to_read = remaining_bits_in_byte.min(n - bits_read);
            let mask = u8::MAX >> (8 - bits_to_read);
            let bits_value =
                (self.data[byte_offset] >> (remaining_bits_in_byte - bits_to_read)) & mask;

            value = (value << bits_to_read) | bits_value as u32;

            bits_read += bits_to_read;
            self.pos += bits_to_read;
            bit_offset = 0;
            byte_offset += 1;
        }
        T::try_from(value).map_err(|_| BitReaderError::InvalidTypeCast)
    }

    // read_ue: unsigned_exp_golomb: https://en.wikipedia.org/wiki/Exponential-Golomb_coding
    pub fn read_ue<T>(&mut self) -> Result<T, BitReaderError>
    where
        T: TryFrom<u32>,
    {
        let mut leading_zero_bits = 0;
        while self.read_bits::<u32>(1)? == 0 {
            leading_zero_bits += 1;
        }

        let mut value = (1 << leading_zero_bits) as u32;
        for i in 0..leading_zero_bits {
            let v = self.read_bits::<u32>(1)? << (leading_zero_bits - i - 1);
            value |= v;
        }
        T::try_from(value - 1).map_err(|_| BitReaderError::InvalidTypeCast)
    }

    pub fn read_se<T>(&mut self) -> Result<T, BitReaderError>
    where
        T: TryFrom<i32>,
    {
        let code_num = self.read_ue::<u32>()? as i32;
        if code_num % 2 == 0 {
            let a = -(code_num / 2);
            return T::try_from(a).map_err(|_| BitReaderError::InvalidTypeCast);
        }
        let a = (code_num / 2) + 1;
        T::try_from(a).map_err(|_| BitReaderError::InvalidTypeCast)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_bits() -> anyhow::Result<()> {
        let data = vec![0b10101010, 0b11001100]; // 예시 데이터
        let mut reader = BitReader::new(&data);

        let value: u8 = reader.read_bits(8)?; // 첫 바이트(8비트)를 읽음
        assert_eq!(value, 0b10101010);

        let value: u8 = reader.read_bits(4)?; // 두 번째 바이트에서 4비트 읽음
        assert_eq!(value, 0b1100);

        let value: u8 = reader.read_bits(4)?; // 나머지 4비트 읽음
        assert_eq!(value, 0b1100);

        Ok(())
    }

    #[test]
    fn test_read_ue() -> anyhow::Result<()> {
        let data = vec![0b00100110]; // 00010 0110 -> leading zeros 3, then "010"
        let mut reader = BitReader::new(&data);

        let value: u32 = reader.read_ue()?; // Exp-Golomb (ue) 값을 읽음
        assert_eq!(value, 3); // 기대 값은 2 (binary: 000100)

        Ok(())
    }

    #[test]
    fn test_read_se() -> anyhow::Result<()> {
        // 00101
        let data = vec![0b10100110, 0b01000010, 0b10000000]; // 0
        let mut reader = BitReader::new(&data);

        let value: i32 = reader.read_se::<i32>()?; // Signed Exp-Golomb (se) 값을 읽음
        assert_eq!(value, 0);

        let value: i32 = reader.read_se::<i32>()?; // Signed Exp-Golomb (se) 값을 읽음
        assert_eq!(value, 1);

        let value: i32 = reader.read_se::<i32>()?; // Signed Exp-Golomb (se) 값을 읽음
        assert_eq!(value, -1);

        let value: i32 = reader.read_se::<i32>()?; // Signed Exp-Golomb (se) 값을 읽음
        assert_eq!(value, 2);

        let value: i32 = reader.read_se::<i32>()?; // Signed Exp-Golomb (se) 값을 읽음
        assert_eq!(value, -2);
        Ok(())
    }

    // #[test]
    // fn test_read_se_positive() {
    //     let data = vec![0b00010100]; // 00010 100 -> leading zeros 2, then "100"
    //     let mut reader = BitReader::new(&data);
    //
    //     let value: i32 = reader.read_se(); // Signed Exp-Golomb (se) 값을 읽음
    //     assert_eq!(value, 2); // 기대 값은 2 (binary: 00010 -> codeNum 3 -> SE 2)
    // }
}
