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
use crate::serial::*;

static mut MILLIS: u64 = 0;
pub static mut TIMERS: [u64; 4] = [0, 0, 0, 0];

// initialize all the PITs
pub fn init() {

}

//
#[inline(always)] // might remove later if issues
fn tick() {
    unsafe {MILLIS += 1;}
}

#[inline(always)]
pub fn get_millis() -> u64 {
    unsafe {MILLIS}
}



