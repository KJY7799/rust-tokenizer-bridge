use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::OnceLock;
use tokenizers::tokenizer::Tokenizer;

// 전역 토크나이저 (메모리에 1회만 상주)
static TOKENIZER: OnceLock<Tokenizer> = OnceLock::new();

#[no_mangle]
pub extern "C" fn init_tokenizer(json_path: *const c_char) -> bool {
    if json_path.is_null() {
        return false;
    }

    let c_json_path = unsafe { CStr::from_ptr(json_path) }.to_string_lossy();

    // 파일로부터 토크나이저 로드
    let tokenizer = match Tokenizer::from_file(c_json_path.as_ref()) {
        Ok(t) => t,
        Err(_) => return false,
    };

    // 전역 변수에 등록 (이미 등록되어 있으면 false 반환)
    TOKENIZER.set(tokenizer).is_ok()
}

#[no_mangle]
pub extern "C" fn encode_to_ids(
    text: *const c_char,
    out_len: *mut usize,
) -> *mut i32 {
    // 1. out_len 안전성 체크 및 초기화
    if out_len.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { *out_len = 0; }

    // 2. 입력 텍스트 및 토크나이저 초기화 여부 체크
    if text.is_null() {
        return std::ptr::null_mut();
    }

    let tokenizer = match TOKENIZER.get() {
        Some(t) => t,
        None => return std::ptr::null_mut(), // init_tokenizer가 먼저 호출되어야 함
    };

    // 3. 인코딩 로직
    let c_text = unsafe { CStr::from_ptr(text) }.to_string_lossy();
    let encoding = match tokenizer.encode(c_text.as_ref(), true) {
        Ok(e) => e,
        Err(_) => return std::ptr::null_mut(),
    };

    // 4. i32 변환 및 오버플로우 체크
    let mut ids = Vec::with_capacity(encoding.get_ids().len());
    for &id in encoding.get_ids() {
        if id > i32::MAX as u32 {
            return std::ptr::null_mut();
        }
        ids.push(id as i32);
    }

    let boxed_slice = ids.into_boxed_slice();
    let len = boxed_slice.len();

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
    fn test_full_process() {
        let path = CString::new("tokenizer.json").unwrap();
        
        // 1. 초기화
        init_tokenizer(path.as_ptr()); 

        // 2. 인코딩 테스트
      let text = CString::new("Hello tokenizer test.").unwrap();
        let mut out_len: usize = 0;

        let ptr = encode_to_ids(text.as_ptr(), &mut out_len as *mut usize);
        
        if !ptr.is_null() {
            // --- 이 부분을 추가했습니다 ---
            println!("Singleton engine check passed");
            println!("Token count: {}", out_len);
            // ---------------------------
            
            assert!(out_len > 0);
            free_ids(ptr, out_len);
            println!("Memory freed successfully");
        } else {
            panic!("Encoding returned NULL");
        }
    }
}