# Rust Tokenizer Bridge (for Intel Mac)

이 프로젝트는 **BGE-M3** 모델의 토크나이징을 Rust 네이티브 환경에서 수행하고, 그 결과를 Java(JNA)에서 사용할 수 있도록 연결하는 브릿지 라이브러리입니다.

---

## 🛠️ Build Instructions (Intel Mac)

PC(Intel Mac)에서 아래 명령어를 실행하여 `.dylib` 파일을 생성하십시오.

### 1. 인텔 맥 타겟 추가 (최초 1회 필요)
```bash
rustup target add x86_64-apple-darwin
```

### 2. 릴리즈 빌드 수행
```bash
cargo build --release --target x86_64-apple-darwin
```

* **Output Path**: `target/x86_64-apple-darwin/release/librust_tokenizer_bridge.dylib`

---

## 📑 API Reference

### 1. `encode_to_ids`
입력 텍스트를 토큰 ID 배열로 변환합니다.
* **`json_path`**: `tokenizer.json` 파일의 경로 (`String`)
* **`text`**: 변환할 입력 문장 (`String`)
* **`out_len`**: 결과 배열의 길이를 받아올 포인터 (`IntByReference`)
* **Return**: 토큰 ID 배열의 메모리 주소 (`Pointer`)

### 2. `free_ids`
Rust에서 할당된 메모리를 명시적으로 해제합니다. (메모리 누수 방지용)
* **`ptr`**: 해제할 배열의 시작 주소 (`Pointer`)
* **`len`**: 배열의 길이 (`int`)

---

## ☕ Java (JNA) Integration Example

```java
import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;
import com.sun.jna.ptr.IntByReference;

public interface RustTokenizer extends Library {
    // 1. 라이브러리 로드 (dylib 파일이 있는 경로 설정)
    RustTokenizer INSTANCE = Native.load("rust_tokenizer_bridge", RustTokenizer.class);

    // 2. Rust 함수 매핑
    Pointer encode_to_ids(String jsonPath, String text, IntByReference outLen);
    void free_ids(Pointer ptr, int len);
}

// --- 실제 사용 예시 ---
public void tokenize() {
    IntByReference outLen = new IntByReference();
    String jsonPath = "./tokenizer.json"; // 실행 위치 기준 경로
    
    // Rust 함수 호출 (주소 반환)
    Pointer ptr = RustTokenizer.INSTANCE.encode_to_ids(jsonPath, "안녕하세요", outLen);
    
    if (ptr != null) {
        int length = outLen.getValue(); // Rust가 기록해준 길이 확인
        int[] tokenIds = ptr.getIntArray(0, length); // 메모리에서 데이터 복사
        
        // 데이터 사용 후 반드시 메모리 해제 호출 (중요)
        RustTokenizer.INSTANCE.free_ids(ptr, length);
    }
}
```

---

## ⚠️ Notes
* **메모리 관리**: Java에서 데이터를 읽어온 후 반드시 `free_ids`를 호출해야 메모리 누수가 발생하지 않습니다.
* **파일 경로**: `json_path`는 현재 작업 디렉토리 기준의 상대 경로 혹은 절대 경로를 모두 지원합니다.
* **플랫폼**: 본 프로젝트는 Intel Mac(`x86_64-apple-darwin`) 환경에서의 빌드 및 실행을 타겟으로 합니다.
* **[업데이트 권장] 성능 최적화 (Singleton)**: 현재는 호출마다 모델을 로드하는 방식이나, 향후 `OnceLock`을 이용해 토크나이저를 전역 메모리에 1회만 상주시키는 패턴으로 변경할 예정입니다. 이렇게 하면 파일 I/O 오버헤드가 제거되어 실제 운영 환경에서 훨씬 빠른 성능을 보장할 수 있습니다.