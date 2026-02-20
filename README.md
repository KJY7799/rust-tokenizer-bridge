# 🚀 Rust Tokenizer Bridge (for Intel Mac)

이 프로젝트는 최신 AI 엔진들의 인텔 맥 지원 중단 문제를 해결하기 위해, HuggingFace 표준 모델의 토크나이징을 Rust 네이티브 환경에서 수행하고 Java(JNA)에서 사용할 수 있도록 연결하는 브릿지 라이브러리입니다.

## 🛠️ 주요 특징
- **인텔 맥(x86_64) 전용**: GitHub Actions(macOS) 서버에서 x86_64 타겟으로 직접 빌드하여 호환성을 확보했습니다.
- **성능 최적화**: `OnceLock` 싱글톤 패턴으로 모델 로드 부하를 최소화했습니다.
- **범용 엔진**: `tokenizer.json` 파일만 교체하면 BGE-M3 외 다른 모델에도 즉시 대응 가능합니다.
- **메모리 관리**: Java에서 호출 후 Rust 메모리를 안전하게 해제할 수 있는 `free_ids` 기능을 포함합니다.

## 📂 파일 구성
- `librust_tokenizer_bridge.dylib`: 인텔 맥용 바이너리 (1.58MB)

## ☕ Java (JNA) 연동 가이드

### 1. 인터페이스 정의 (Java)
자바 프로젝트 내에 아래 인터페이스를 생성하여 네이티브 함수를 매핑합니다.

```java
import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;
import com.sun.jna.ptr.IntByReference;

public interface RustTokenizer extends Library {
    // .dylib 파일명이 librust_tokenizer_bridge일 경우
    RustTokenizer INSTANCE = Native.load("rust_tokenizer_bridge", RustTokenizer.class);

    boolean init_tokenizer(String jsonPath);
    Pointer encode_to_ids(String text, IntByReference outLen);
    void free_ids(Pointer ptr, int len);
}
```
### 2. 실제 사용 예시
```java
// 1. 초기화 (서버 기동 시 1회)
if (!RustTokenizer.INSTANCE.init_tokenizer("./tokenizer.json")) {
    throw new RuntimeException("초기화 실패");
}

// 2. 실행
IntByReference outLen = new IntByReference();
Pointer ptr = RustTokenizer.INSTANCE.encode_to_ids("안녕하세요", outLen);

if (ptr != null) {
    int length = outLen.getValue();
    int[] tokenIds = ptr.getIntArray(0, length); // 결과 복사
    
    // 3. 메모리 해제 (필수)
    RustTokenizer.INSTANCE.free_ids(ptr, length);
}
```

실행 시 주의사항

보안 차단: 맥에서 실행 시 "확인되지 않은 개발자" 메시지가 뜨면 아래 명령어를 터미널에서 실행하세요.

xattr -d com.apple.quarantine librust_tokenizer_bridge.dylib