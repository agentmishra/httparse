use ::Bytes;

pub unsafe fn parse_uri_batch_16<'a>(bytes: &mut Bytes<'a>) {
    while bytes.as_ref().len() >= 16 {
        let advance = match_url_char_16_sse(bytes.as_ref());
        bytes.advance(advance);

        if advance != 16 {
            break;
        }
    }
}

#[target_feature(enable = "sse4.2")]
#[allow(non_snake_case, overflowing_literals)]
unsafe fn match_url_char_16_sse(buf: &[u8]) -> usize {
    debug_assert!(buf.len() >= 16);

    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    let ptr = buf.as_ptr();

    let LSH: __m128i = _mm_set1_epi8(0x0f);
    let URI: __m128i = _mm_setr_epi8(
        0xb8, 0xfc, 0xf8, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc,
        0xfc, 0xfc, 0xfc, 0x7c, 0x54, 0x7c, 0xd4, 0x7c,
    );
    let ARF: __m128i = _mm_setr_epi8(
        0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    );

    let data = _mm_lddqu_si128(ptr as *const _);
    let rbms = _mm_shuffle_epi8(URI, data);
    let cols = _mm_and_si128(LSH, _mm_srli_epi16(data, 4));
    let bits = _mm_and_si128(_mm_shuffle_epi8(ARF, cols), rbms);

    let v = _mm_cmpeq_epi8(bits, _mm_setzero_si128());
    let r = 0xffff_0000 | _mm_movemask_epi8(v) as u32;

    _tzcnt_u32(r) as usize
}

pub unsafe fn match_header_value_batch_16(bytes: &mut Bytes) {
    while bytes.as_ref().len() >= 16 {
        let advance = match_header_value_char_16_sse(bytes.as_ref());
        bytes.advance(advance);

       if advance != 16 {
            break;
       }
    }
}

#[target_feature(enable = "sse4.2")]
#[allow(non_snake_case)]
unsafe fn match_header_value_char_16_sse(buf: &[u8]) -> usize {
    debug_assert!(buf.len() >= 16);

    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    let ptr = buf.as_ptr();

    // %x09 %x20-%x7e %x80-%xff
    let TAB: __m128i = _mm_set1_epi8(0x09);
    let DEL: __m128i = _mm_set1_epi8(0x7f);
    let LOW: __m128i = _mm_set1_epi8(0x1f);

    let dat = _mm_lddqu_si128(ptr as *const _);
    let low = _mm_cmpgt_epi8(dat, LOW);
    let tab = _mm_cmpeq_epi8(dat, TAB);
    let del = _mm_cmpeq_epi8(dat, DEL);
    let bit = _mm_andnot_si128(del, _mm_or_si128(low, tab));
    let rev = _mm_cmpeq_epi8(bit, _mm_setzero_si128());
    let res = 0xffff_0000 | _mm_movemask_epi8(rev) as u32;

    _tzcnt_u32(res) as usize
}
