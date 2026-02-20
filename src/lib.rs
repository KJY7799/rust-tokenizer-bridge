use std::ffi::CStr;
use std::os::raw::c_char;
use tokenizers::tokenizer::Tokenizer;

#[no_mangle]
pub extern "C" fn encode_to_ids(
    json_path: *const c_char,
    text: *const c_char,
    out_len: *mut usize,
) -> *mut i32 {
    // 1. out_len 포인터 자체의 NULL 체크 (쓰기 가능 여부 확인)
    if out_len.is_null() {
        return std::ptr::null_mut();
    }

    // 2. 초기화: 이제 out_len은 안전하므로 즉시 0으로 설정
    // 이렇게 해야 이후 어떤 에러가 나더라도 자바 쪽에서는 0을 읽게 됩니다.
    unsafe { *out_len = 0; }

    // 3. 나머지 입력값 체크
    if json_path.is_null() || text.is_null() {
        return std::ptr::null_mut();
    }

    // 4. C 문자열을 Rust 문자열로 변환
    let c_json_path = unsafe { CStr::from_ptr(json_path) }.to_string_lossy();
    let c_text = unsafe { CStr::from_ptr(text) }.to_string_lossy();

    // 5. 토크나이저 로드
    let tokenizer = match Tokenizer::from_file(c_json_path.as_ref()) {
        Ok(t) => t,
        Err(_) => return std::ptr::null_mut(),
    };

    // 6. 인코딩 수행
    let encoding = match tokenizer.encode(c_text.as_ref(), true) {
        Ok(e) => e,
        Err(_) => return std::ptr::null_mut(),
    };

    // 7. JNI/JNA 호환성 체크 (i32 오버플로우 방지)
    let mut ids = Vec::with_capacity(encoding.get_ids().len());
    for &id in encoding.get_ids() {
        if id > i32::MAX as u32 {
            return std::ptr::null_mut();
        }
        ids.push(id as i32);
    }

    let boxed_slice = ids.into_boxed_slice();
    let len = boxed_slice.len();

    // 8. 최종 결과 기록
    unsafe { *out_len = len; }

    Box::into_raw(boxed_slice) as *mut i32
}

#[no_mangle]
pub extern "C" fn free_ids(ptr: *mut i32, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }

    unsafe {
        let slice_ptr = std::ptr::slice_from_raw_parts_mut(ptr, len);
        drop(Box::from_raw(slice_ptr));
    }
}

// ---------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_full_flow() {
        let json_path = CString::new("tokenizer.json").expect("CString failed");
        let text = CString::new("안녕하세요, 테스트 문장입니다.").expect("CString failed");
        let mut out_len: usize = 0;

        let ptr = encode_to_ids(
            json_path.as_ptr(),
            text.as_ptr(),
            &mut out_len as *mut usize
        );

        if !ptr.is_null() {
            println!("✅ 테스트 성공! 추출된 토큰 개수: {}", out_len);
            assert!(out_len > 0);
            free_ids(ptr, out_len);
        } else {
            panic!("❌ 테스트 실패: 포인터가 NULL입니다.");
        }
    }

    #[test]
    fn test_invalid_input() {
        let mut out_len: usize = 999;
        // out_len 포인터는 정상이지만, 다른 값이 null인 경우를 테스트
        let ptr = encode_to_ids(std::ptr::null(), std::ptr::null(), &mut out_len);
        
        assert!(ptr.is_null());
        assert_eq!(out_len, 0, "실패 시 out_len은 반드시 0으로 업데이트되어야 합니다.");
    }
}