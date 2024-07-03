#[cfg(target_os = "zkvm")]
use core::arch::asm;

/// Adds two Bls12381 points.
///
/// The result is stored in the first point.
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn syscall_bls12381_add(p: *mut u32, q: *const u32) {
    #[cfg(target_os = "zkvm")]
    unsafe {
        asm!(
            "ecall",
            in("t0") crate::syscalls::BLS12381_ADD,
            in("a0") p,
            in("a1") q,
        );
    }

    #[cfg(not(target_os = "zkvm"))]
    unreachable!()
}

/// Double a Bls12381 point.
///
/// The result is stored in the first point.
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn syscall_bls12381_double(p: *mut u32) {
    #[cfg(target_os = "zkvm")]
    unsafe {
        asm!(
            "ecall",
            in("t0") crate::syscalls::BLS12381_DOUBLE,
            in("a0") p,
            in("a1") 0,
        );
    }
}

/// Decompresses a compressed BLS12-381 point.
///
/// The first half of the input array should contain the X coordinate.
/// The second half of the input array will be overwritten with the Y coordinate.
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn syscall_bls12381_decompress(point: &mut [u8; 96], is_odd: bool) {
    #[cfg(target_os = "zkvm")]
    {
        // Memory system/FpOps are little endian so we'll just flip the whole array before/after
        point.reverse();
        let p = point.as_mut_ptr();
        unsafe {
            asm!(
                "ecall",
                in("t0") crate::syscalls::BLS12381_DECOMPRESS,
                in("a0") p,
                in("a1") is_odd as u8,
            );
        }
        point.reverse();
    }

    #[cfg(not(target_os = "zkvm"))]
    unreachable!()
}
