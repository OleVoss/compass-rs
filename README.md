## Questions
### 1. Header parsing
- Bitflags as struct fields? \
Parsing the fields and setting them with a custom `set_field!("CRC", 0)` macro; then `header.CRC`
- keeping the header fields as raw bytes and retrieving with functions? \
Ala `header.crc()` or `header.get_crc()`
```rust
pub fn crc(&self) -> u8 {
	(self.first_byte >> 5) & 1
}
```
- single method? \
Acces fields via bitflags and a retrieval function `header.get(HeaderFields::CRC)` \
```rust
pub fn get(&self, field: HeaderField::CRC) -> u8 {
	(self.first_byte >> 7) & field
		 	 ^^^^---> would't work since the bitshift would be different for every flag
}
```
_Parsing_ implies checking for errors (I guess). Therefore should the bitflags be avaluated and set when the packet is created and the first idea seems reasonable to implement. 
