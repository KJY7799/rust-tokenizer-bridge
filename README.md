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

### 1. `init_tokenizer`
토크나이저를 전역 메모리에 1회 로드합니다. (최초 1회 실행 필수)
* **`json_path`**: `tokenizer.json` 파일의 경로 (`String`)
* **Return**: 초기화 성공 여부 (`boolean`)

### 2. `encode_to_ids`
텍스트를 토큰 ID 배열로 변환합니다. (메모리에 로드된 객체를 사용하여 매우 빠름)
* **`text`**: 변환할 입력 문장 (`String`)
* **`out_len`**: 결과 배열의 길이를 받아올 포인터 (`IntByReference`)
* **Return**: 토큰 ID 배열의 메모리 주소 (`Pointer`)

### 3. `free_ids`
Rust에서 할당된 메모리를 명시적으로 해제합니다.
* **`ptr`**: 해제할 배열의 시작 주소 (`Pointer`)
* **`len`**: 배열의 길이 (`int`)

---

## ☕ Java (JNA) Integration Example

```java
public interface RustTokenizer extends Library {
    RustTokenizer INSTANCE = Native.load("rust_tokenizer_bridge", RustTokenizer.class);

    boolean init_tokenizer(String jsonPath);
    Pointer encode_to_ids(String text, IntByReference outLen);
    void free_ids(Pointer ptr, int len);
}

// --- Usage ---
// 1. 서버 기동 시점에 초기화
if (!RustTokenizer.INSTANCE.init_tokenizer("./tokenizer.json")) {
    throw new RuntimeException("Tokenizer init failed");
}

// 2. 필요 시 인코딩 호출
IntByReference outLen = new IntByReference();
Pointer ptr = RustTokenizer.INSTANCE.encode_to_ids("안녕하세요", outLen);

if (ptr != null) {
    int length = outLen.getValue();
    int[] tokenIds = ptr.getIntArray(0, length);
    RustTokenizer.INSTANCE.free_ids(ptr, length); // 메모리 해제 필수
}
```

---

## ⚠️ Notes
* **메모리 관리**: Java에서 데이터를 읽어온 후 반드시 `free_ids`를 호출해야 메모리 누수가 발생하지 않습니다.
* **성능 최적화**: `OnceLock` 싱글톤 패턴을 적용하여 파일 I/O 부하를 최소화했습니다.
* **플랫폼**: 본 프로젝트는 Intel Mac(`x86_64-apple-darwin`) 환경을 타겟으로 빌드해야 정상 작동합니다.