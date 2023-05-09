/* module for generating interrupts every millis
(hopefully "async") so we can time stuff in a
somewhat reliable fashion.
hopefully it will be 
 */

/* notes 

    the "natural" frequency of the timer is 1193181.18 Hz
    so if you want to create a millis interrupt,
    it should be once every 1193 ticks...

*/
use crate::tooling::serial::*;

const DIVISOR: u16 = 1193;  // == 1193181 / 1000 hz

pub static mut MILLIS: u64 = 0;
pub static mut TIMER: Timer = Timer {
    max: 500,
    cur: 0,
    func: &say_hi,
    active: false
};

#[repr(C)]
#[derive(Clone)]
pub struct Timer<'a> {
    max: u64,
    cur: u64,
    func: &'a dyn Fn(),
    active: bool
}

// set the PIT to 1000 interrupts/sec, instead of the default 18
pub fn init() {
    outb(0x43, 0b00001100);
    outb(0x40, (DIVISOR & 0xff) as u8); // put low byte of DIVISOR in 0x40
    outb(0x40, ((DIVISOR >> 8) & 0xff) as u8); // put high byte

    outb(0x21, inb(0x21) & 0xFE);  // chat gippity, i have no idea wtf this does
                                                    // however very ugly panic if this line is omitted
}

#[inline]
pub fn get_millis() -> u64 {
    unsafe {MILLIS}
}

// yeah this rust syntax makes me want to vomit
// but basically you can pass a fn into Timer and it will be executed when the timer is done
// only empty void functions are supported atm, hopefully somebody who knows rust can add params and return value (wrapped in Option?)
impl<'a> Timer<'a> {

    pub fn new(time: u64, function: &'a dyn Fn()) -> Self {
        Timer {
            max: time,
            cur: 0,
            func: function,
            active: false
        }
    }

    // (re)start the timer
    pub fn init(&mut self) {
        self.active = true;
        self.cur = 0;
    }

    // stop the timer
    pub fn stop(&mut self) {
        self.active = false;
    }

    pub fn tick(&mut self){
        if self.active {
            self.cur += 1;
            if self.cur >= self.max {
                (self.func)();
                // self.active = false;
            }
        }
    }

}

pub fn say_hi() {
    println!("A random number between 1 and 100: {}", unsafe {crate::misc::rand::RNG.normalized_f64()});
    unsafe {TIMER.init();}
}


