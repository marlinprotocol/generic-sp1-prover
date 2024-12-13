use std::marker::PhantomData;

use sp1_curves::{CurveType, EllipticCurve};

use crate::{
    events::create_ec_decompress_event,
    syscalls::{Syscall, SyscallContext},
};

pub(crate) struct WeierstrassDecompressSyscall<E: EllipticCurve> {
    _phantom: std::marker::PhantomData<E>,
}

impl<E: EllipticCurve> WeierstrassDecompressSyscall<E> {
    /// Create a new instance of the [`WeierstrassDecompressSyscall`].
    pub const fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}

impl<E: EllipticCurve> Syscall for WeierstrassDecompressSyscall<E> {
    fn execute(&self, rt: &mut SyscallContext, arg1: u32, arg2: u32) -> Option<u32> {
        let event = create_ec_decompress_event::<E>(rt, arg1, arg2);
        match E::CURVE_TYPE {
            CurveType::Secp256k1 => rt.record_mut().k256_decompress_events.push(event),
            CurveType::Bls12381 => rt.record_mut().bls12381_decompress_events.push(event),
            _ => panic!("Unsupported curve"),
        }
        None
    }

    fn num_extra_cycles(&self) -> u32 {
        0
    }
}
