use num::rational::Ratio;

#[derive(Debug, Clone)]
pub struct Rational {
    pub num: i32,
    pub den: i32,
}

impl Rational {
    pub fn new(num: i32, den: i32) -> Self {
        Self { num, den }
    }
}

pub fn rescale(value: i64, src: Rational, dst: Rational) -> Option<i64> {
    let numerator = value
        .checked_mul(src.num as i64)?
        .checked_mul(dst.den as i64)?;
    let denominator = (src.den as i64).checked_mul(dst.num as i64)?;

    // 분모이 0이 아닌지 확인
    if denominator == 0 {
        return None;
    }

    Some(numerator / denominator)
}

pub fn rescale_with_rounding(value: i64, src: &Rational, dst: &Rational) -> Option<i64> {
    if src.den == 0 || dst.den == 0 {
        return None;
    }

    let src_ratio = Ratio::new(src.num as i64, src.den as i64);
    let dst_ratio = Ratio::new(dst.num as i64, dst.den as i64);

    let value_ratio = Ratio::from_integer(value);

    // 스케일링: value * src / dst
    let scaled = value_ratio * src_ratio / dst_ratio;

    // 반올림: 가장 가까운 정수로 반올림
    let rounded = scaled.round();

    // i64로 변환
    rounded.to_integer();

    Some(rounded.to_integer())
}
