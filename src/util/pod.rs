/*!
Podness.
*/

/// Defines types which can be safely `transmute`d from any byte pattern.
///
/// Types which need to be read from PE files should implement this.
///
/// # Safety
///
/// Various functions rely on `Pod`ness to mean that any byte array of sufficient length can be safely transmuted to this type.
pub unsafe trait Pod: 'static {}

// Autoderive `Pod`ness
// unsafe impl Pod for .. {}

unsafe impl Pod for i8 {}
unsafe impl Pod for i16 {}
unsafe impl Pod for i32 {}
unsafe impl Pod for i64 {}

unsafe impl Pod for u8 {}
unsafe impl Pod for u16 {}
unsafe impl Pod for u32 {}
unsafe impl Pod for u64 {}

unsafe impl Pod for f32 {}
unsafe impl Pod for f64 {}

macro_rules! impl_pod_array {
	($n:tt $($tail:tt)+) => {
		unsafe impl<T: Pod> Pod for [T; $n] {}
		impl_pod_array!($($tail)+);
	};
	($n:tt) => {
		unsafe impl<T: Pod> Pod for [T; $n] {}
	};
}
impl_pod_array!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15);
impl_pod_array!(16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31);
impl_pod_array!(32 48 64 80 100 128 160 192 256 512 768 1024 2048 4096);
