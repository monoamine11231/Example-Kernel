// pseudorandom number generator
use crate::time::get_millis;
const FLOAT_DIVISOR: f32 = 8_388_607.0;
const DOUBLE_DIVISOR: f64 = 9_007_199_254_740_991.0;
const FLOAT_DIVISOR_U32: u32 = 8388607;
const DOUBLE_DIVISOR_U64: u64 = 9007199254740991;
pub static mut RNG: Rng = Rng::test();

pub struct Rng {
    current: u64,
}

impl Rng {
    // initialize the random number generator
    pub fn new() -> Self {
        Rng {
            current: get_millis() + 11111111,
        }
    }

    pub const fn test() -> Self {
        Rng { current: 11111111 }
    }

    // use the xorshift algo to generate the next number
    #[inline]
    fn next(&mut self) {
        self.current ^= self.current << 13;
        self.current ^= self.current >> 7;
        self.current ^= self.current << 17;
        println!("My current is {} now!", self.current);
    }

    // generate a random number

    pub fn u32(&mut self) -> u32 {
        self.next();
        self.current as u32
    }

    pub fn i32(&mut self) -> i32 {
        i32::from_le_bytes(self.u32().to_le_bytes())
    }

    pub fn u64(&mut self) -> u64 {
        self.next();
        self.current
    }

    pub fn i64(&mut self) -> i64 {
        i64::from_le_bytes(self.u64().to_le_bytes())
    }

    // take the low 23 bits of self.current and divide by (float) 2**23 - 1
    // which is the highest float with a distance <= 1.0 to the next float
    // not sure if normalized is the right name but it's the same as Random.random() in other langs
    pub fn normalized_f32(&mut self) -> f32 {
        self.next();
        ((self.current as u32) & 8_388_607) as f32 / FLOAT_DIVISOR
    }

    pub fn normalized_f64(&mut self) -> f64 {
        self.next();
        ((self.current) & 9_007_199_254_740_991) as f64 / DOUBLE_DIVISOR
    }

    // NOTE: f32() and f64() only return positive numbers

    // trust me bro
    /*     pub fn f32(&mut self) -> f32 {
        let exponent = (self.u32() & 255).saturating_sub(1);
        f32::from_bits(
            self.u32()
            & 8_388_607
            + exponent << 23
        )
    }

    pub fn f64(&mut self) -> f64 {
        let exponent = (self.u64() & 1023).saturating_sub(1);
        f64::from_bits(
            self.u64()
            & 9_007_199_254_740_991
            + exponent << 52
        )
    } */

    pub fn range(&mut self, a: i64, b: i64) -> i64 {
        a + (self.i64() % (a + b))
    }
}
