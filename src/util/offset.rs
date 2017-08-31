
/// Offset operator for address types.
pub trait Offset<I> {
	fn offset(self, int: I, size_of: usize) -> Self;
}

// Offset 32-bit Va.
use pe32::Va as Va32;
impl Offset<i32> for Va32 { fn offset(self, int: i32, size_of: usize) -> Va32 { self.wrapping_add(int as Va32 * size_of as Va32) } }
impl Offset<u32> for Va32 { fn offset(self, int: u32, size_of: usize) -> Va32 { self.wrapping_add(int as Va32 * size_of as Va32) } }
impl Offset<isize> for Va32 { fn offset(self, int: isize, size_of: usize) -> Va32 { self.wrapping_add(int as Va32 * size_of as Va32) } }
impl Offset<usize> for Va32 { fn offset(self, int: usize, size_of: usize) -> Va32 { self.wrapping_add(int as Va32 * size_of as Va32) } }

// Offset 64-bit Va.
use pe64::Va as Va64;
impl Offset<i32> for Va64 { fn offset(self, int: i32, size_of: usize) -> Va64 { self.wrapping_add(int as Va64 * size_of as Va64) } }
impl Offset<u32> for Va64 { fn offset(self, int: u32, size_of: usize) -> Va64 { self.wrapping_add(int as Va64 * size_of as Va64) } }
impl Offset<i64> for Va64 { fn offset(self, int: i64, size_of: usize) -> Va64 { self.wrapping_add(int as Va64 * size_of as Va64) } }
impl Offset<u64> for Va64 { fn offset(self, int: u64, size_of: usize) -> Va64 { self.wrapping_add(int as Va64 * size_of as Va64) } }
impl Offset<isize> for Va64 { fn offset(self, int: isize, size_of: usize) -> Va64 { self.wrapping_add(int as Va64 * size_of as Va64) } }
impl Offset<usize> for Va64 { fn offset(self, int: usize, size_of: usize) -> Va64 { self.wrapping_add(int as Va64 * size_of as Va64) } }
