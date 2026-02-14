#ifndef SM_STRING_UTILS_H
#define SM_STRING_UTILS_H

#include "freestanding.h"

size_t sm_strlen(const char* s);

void sm_int_to_string(int64_t num, char* buf);
void sm_uint64_to_string(uint64_t num, char* buf);
void sm_int128_to_string(int128_t num, char* buf);
void sm_uint128_to_string(uint128_t num, char* buf);

int sm_parse_i8(const char* str, int8_t* out);
int sm_parse_u8(const char* str, uint8_t* out);
int sm_parse_i16(const char* str, int16_t* out);
int sm_parse_u16(const char* str, uint16_t* out);
int sm_parse_i32(const char* str, int32_t* out);
int sm_parse_u32(const char* str, uint32_t* out);
int sm_parse_i64(const char* str, int64_t* out);
int sm_parse_u64(const char* str, uint64_t* out);

#endif