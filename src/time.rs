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

const DIVISOR: u16 = 1193;  // == 1193182 / 1000 hz

pub static mut MILLIS: u64 = 0;
pub static mut TIMERS: [u64; 4] = [0, 0, 0, 0];

#[repr(C)]
pub struct Timer<'a, A, R>
{
    max: u64,
    cur: u64,
    func: &'a dyn Fn(A) -> R
}

// set the PIT to 1000 interrupts/sec, instead of the default 18
pub fn init() {
    outb(0x43, 0b00001100);
    outb(0x40, (DIVISOR & 0xff) as u8); // put low byte of DIVISOR in 0x40
    outb(0x40, ((DIVISOR >> 8) & 0xff) as u8); // put high byte

    outb(0x21, inb(0x21) & 0xFE);  // chat gippity, i have no idea wtf this does
                                                    // however very ugly panic if this line is omitted
}

// yeah this rust syntax makes me want to vomit
// but basically you can pass a fn into Timer with its args, and it will be executed when the timer is done
impl<'a, A, R> Timer<'a, A, R> {

    fn new(time: u64, function: &'a dyn Fn(A) -> R) -> Self
    {
        Timer {
            max: time,
            cur: 0,
            func: function
        }
    }

}


