use std::ffi::CStr;
use std::os::raw::c_char;
use tokenizers::Tokenizer;

#[no_mangle]
pub extern "C" fn encode_to_ids(json_path: *const c_char, text: *const c_char, out_len: *mut usize) -> *mut i32 {
    // 1. 입력값 검증 및 변환
    if json_path.is_null() || text.is_null() { return std::ptr::null_mut(); }

    let c_path = unsafe { CStr::from_ptr(json_path) }.to_str().unwrap_or("");
    let c_text = unsafe { CStr::from_ptr(text) }.to_str().unwrap_or("");

    // 2. 토크나이저 실행
    let tokenizer = match Tokenizer::from_file(c_path) {
        Ok(t) => t,
        Err(_) => return std::ptr::null_mut(),
    };

    let encoding = match tokenizer.encode(c_text, true) {
        Ok(e) => e,
        Err(_) => return std::ptr::null_mut(),
    };

    // 3. 결과 데이터를 i32 벡터로 변환
    let mut ids: Vec<i32> = encoding.get_ids().iter().map(|&x| x as i32).collect();
    
    // 4. 중요: 자바에 길이를 알려줌
    unsafe { *out_len = ids.len(); }

    // 5. 핵심: Rust의 메모리 관리 해제 (Box::into_raw)    
    let ptr = ids.as_mut_ptr();
    std::mem::forget(ids); 

    ptr
}

// 6. 메모리 해제 함수 (자바에서 다 쓰고 이걸 꼭 호출해줘야 메모리 누수가 안 납니다)
#[no_mangle]
pub extern "C" fn free_ids(ptr: *mut i32, len: usize) {
    if !ptr.is_null() {
        unsafe { Vec::from_raw_parts(ptr, len, len); } // 다시 Rust가 관리하게 해서 드랍시킴
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding() {
        // 1. 실제 tokenizer.json 파일이 있는 경로를 적으세요 (테스트용)
        let json_path = std::ffi::CString::new("./tokenizer.json").unwrap();
        let test_text = std::ffi::CString::new("안녕하세요 반갑습니당").unwrap();
        let mut out_len: usize = 0;

        // 2. 우리가 만든 함수 호출
        let ptr = encode_to_ids(json_path.as_ptr(), test_text.as_ptr(), &mut out_len);

        // 3. 결과 확인
        assert!(!ptr.is_null(), "포인터가 비어있습니다 (로드 실패)");
        assert!(out_len > 0, "토큰이 생성되지 않았습니다");

        println!("테스트 성공! 토큰 개수: {}, 포인터 주소: {:?}", out_len, ptr);

        // 4. 메모리 해제 테스트
        free_ids(ptr, out_len);
    }
}