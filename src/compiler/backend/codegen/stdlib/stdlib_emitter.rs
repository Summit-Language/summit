use super::super::emitter::CEmitter;

/// Generates C declarations for standard library functions.
pub struct StdlibEmitter;

impl StdlibEmitter {
    /// Creates a new StdlibEmitter instance.
    pub fn new() -> Self {
        StdlibEmitter
    }

    /// Emits a C declaration for the specified standard library function.
    ///
    /// # Parameters
    /// - `self`: Immutable reference to self
    /// - `emitter`: The C emitter to write declarations to
    /// - `func_name`: The name of the standard library function
    pub fn emit_decl(&self, emitter: &mut CEmitter, func_name: &str) {
        match func_name {
            "sm_std_io_println" => emitter
                .emit_line("void sm_std_io_println(const char* s);"),
            "sm_std_io_print" => emitter
                .emit_line("void sm_std_io_print(const char* s);"),
            "sm_std_io_println_bool" => emitter
                .emit_line("void sm_std_io_println_bool(bool b);"),
            "sm_std_io_print_bool" => emitter
                .emit_line("void sm_std_io_print_bool(bool b);"),
            "sm_std_io_println_i64" => emitter
                .emit_line("void sm_std_io_println_i64(int64_t n);"),
            "sm_std_io_print_i64" => emitter
                .emit_line("void sm_std_io_print_i64(int64_t n);"),
            "sm_std_io_println_u64" => emitter
                .emit_line("void sm_std_io_println_u64(uint64_t n);"),
            "sm_std_io_print_u64" => emitter
                .emit_line("void sm_std_io_print_u64(uint64_t n);"),
            "sm_std_io_println_i128" => emitter
                .emit_line("void sm_std_io_println_i128(__int128 n);"),
            "sm_std_io_print_i128" => emitter
                .emit_line("void sm_std_io_print_i128(__int128 n);"),
            "sm_std_io_println_u128" => emitter
                .emit_line("void sm_std_io_println_u128(unsigned __int128 n);"),
            "sm_std_io_print_u128" => emitter
                .emit_line("void sm_std_io_print_u128(unsigned __int128 n);"),
            "sm_std_io_readln" => emitter
                .emit_line("const char* sm_std_io_readln(void);"),
            "sm_std_io_read_i8" => emitter
                .emit_line("int8_t sm_std_io_read_i8(void);"),
            "sm_std_io_read_u8" => emitter
                .emit_line("uint8_t sm_std_io_read_u8(void);"),
            "sm_std_io_read_i16" => emitter
                .emit_line("int16_t sm_std_io_read_i16(void);"),
            "sm_std_io_read_u16" => emitter
                .emit_line("uint16_t sm_std_io_read_u16(void);"),
            "sm_std_io_read_i32" => emitter
                .emit_line("int32_t sm_std_io_read_i32(void);"),
            "sm_std_io_read_u32" => emitter
                .emit_line("uint32_t sm_std_io_read_u32(void);"),
            "sm_std_io_read_i64" => emitter
                .emit_line("int64_t sm_std_io_read_i64(void);"),
            "sm_std_io_read_u64" => emitter
                .emit_line("uint64_t sm_std_io_read_u64(void);"),
            _ => {}
        }
    }
}