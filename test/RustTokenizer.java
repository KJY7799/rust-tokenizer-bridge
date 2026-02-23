import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;
import com.sun.jna.ptr.IntByReference;

public interface RustTokenizer extends Library {
  
    RustTokenizer INSTANCE =
        Native.load("rust_tokenizer_bridge", RustTokenizer.class);

    boolean init_tokenizer(String jsonPath);
    Pointer encode_to_ids(String text, IntByReference outLen);
    void free_ids(Pointer ptr, int len);
}