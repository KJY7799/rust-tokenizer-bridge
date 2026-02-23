import com.sun.jna.Pointer;
import com.sun.jna.ptr.IntByReference;

public class TestMain {
    public static void main(String[] args) {

        System.out.println("init_tokenizer");
        boolean ok =
            RustTokenizer.INSTANCE.init_tokenizer("tokenizer.json");
        if (!ok) {
            throw new RuntimeException("init_tokenizer FAILED");
        }

        System.out.println("encode_to_ids");
        IntByReference outLen = new IntByReference();
        Pointer ptr =
            RustTokenizer.INSTANCE.encode_to_ids("Windows JNA Test", outLen);

        if (ptr == Pointer.NULL) {
            throw new RuntimeException("encode_to_ids returned NULL");
        }

        int len = outLen.getValue();
        System.out.println("Token count = " + len);

        int[] ids = ptr.getIntArray(0, len);
        System.out.print("IDs: ");
        for (int id : ids) {
            System.out.print(id + " ");
        }
        System.out.println();

        System.out.println("free_ids");
        RustTokenizer.INSTANCE.free_ids(ptr, len);

        System.out.println("JNA TEST SUCCESS");
    }
}