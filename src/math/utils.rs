use core::{
    arch::asm,
    ptr::{addr_of, addr_of_mut},
};

#[inline(always)]
pub fn sqrt(x: f32) -> f32 {
    let mut x_ptr = addr_of!(x) as u64;

    let result: f32;
    unsafe {
        asm!(
            "
            mov rax, {0}
            fld dword ptr [rax]
            fsqrt
            fstp dword ptr [rax]
            ", in(reg) x_ptr
        );
    }
    return x;
}

pub fn float_ceil(a : f32) -> usize{
    return (a + 0.5) as usize;
}
pub fn float_floor(a : f32) -> usize{
    return (a - 0.5) as usize;
}
