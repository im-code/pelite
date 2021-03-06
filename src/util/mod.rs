/*!
Utilities and other tidbits.
*/

#[macro_use]
mod iter;

mod c_str;
mod wide_str;
mod pod;
mod offset;
mod slice_len;

pub use self::c_str::CStr;
pub use self::wide_str::WideStr;
pub use self::pod::Pod;
pub use self::offset::Offset;
pub use self::slice_len::SliceLen;

/// Splits a slice at the point defined by the callback.
#[inline]
pub(crate) fn split_f<T, F: FnMut(&T) -> bool>(slice: &[T], f: F) -> (&[T], &[T]) {
	let i = slice.iter().position(f).unwrap_or(slice.len());
	(&slice[..i], &slice[i..])
}

/// Reads an optionally nul-terminated string from byte buffer.
///
/// Returns the slice split before the nul byte and the whole slice if no nul byte is found.
///
/// Analog to the `strn*` family of C string functions.
///
/// # Examples
///
/// ```
/// use pelite::util::strn;
///
/// let buf: &[u8; 8] = b"STRING\0\0";
/// assert_eq!(strn(buf), b"STRING");
///
/// let buf: &[u8; 4] = b"FOUR";
/// assert_eq!(strn(buf), b"FOUR");
/// ```
#[inline]
pub fn strn(buf: &[u8]) -> &[u8] {
	split_f(buf, |&byte| byte == 0).0
}

/// Reads an optionally nul-terminated wide char string from buffer.
///
/// Returns the slice split before the nul word and the whole slice if no nul word is found.
///
/// Analog to the `wcsn*` family of C string functions.
///
/// # Examples
///
/// ```
/// use pelite::util::wstrn;
///
/// let buf: [u16; 8] = [83, 84, 82, 73, 78, 71, 0, 0];
/// assert_eq!(wstrn(&buf), &[83, 84, 82, 73, 78, 71]);
///
/// let buf: [u16; 4] = [70, 79, 85, 82];
/// assert_eq!(wstrn(&buf), &[70, 79, 85, 82]);
/// ```
#[inline]
pub fn wstrn(buf: &[u16]) -> &[u16] {
	split_f(buf, |&word| word == 0).0
}
