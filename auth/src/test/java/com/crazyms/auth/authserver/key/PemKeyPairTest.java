package com.crazyms.auth.authserver.key;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import com.nimbusds.jose.jwk.Curve;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.KeyPair;
import java.security.KeyPairGenerator;
import java.security.spec.ECGenParameterSpec;
import java.util.Base64;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

class PemKeyPairTest {

    @TempDir
    Path dir;

    @Test
    void loadsAMatchingP256Pair() throws Exception {
        var pair = generate("secp256r1");

        var key = PemKeyPair.load(write("priv.pem", "PRIVATE KEY", pair.getPrivate().getEncoded()),
                write("pub.pem", "PUBLIC KEY", pair.getPublic().getEncoded()));

        assertThat(key.getCurve()).isEqualTo(Curve.P_256);
        assertThat(key.isPrivate()).isTrue();
        assertThat(key.getKeyID()).isEqualTo(key.computeThumbprint().toString());
    }

    @Test
    void rejectsAPrivateKeyThatDoesNotMatchThePublicKey() throws Exception {
        var a = generate("secp256r1");
        var b = generate("secp256r1");

        assertThatThrownBy(() -> PemKeyPair.load(write("priv.pem", "PRIVATE KEY", a.getPrivate().getEncoded()),
                        write("pub.pem", "PUBLIC KEY", b.getPublic().getEncoded())))
                .isInstanceOf(IllegalStateException.class)
                .hasMessageContaining("do not match");
    }

    @Test
    void rejectsKeysOnOtherCurves() throws Exception {
        var pair = generate("secp384r1");

        assertThatThrownBy(() -> PemKeyPair.load(write("priv.pem", "PRIVATE KEY", pair.getPrivate().getEncoded()),
                        write("pub.pem", "PUBLIC KEY", pair.getPublic().getEncoded())))
                .isInstanceOf(IllegalStateException.class)
                .hasMessageContaining("P-256");
    }

    @Test
    void rejectsFilesThatAreNotPem() throws Exception {
        Path garbage = Files.writeString(dir.resolve("garbage.pem"), "not a key");

        assertThatThrownBy(() -> PemKeyPair.load(garbage, garbage))
                .isInstanceOf(IllegalStateException.class)
                .hasMessageContaining("is not a PEM");
    }

    private static KeyPair generate(String curve) throws Exception {
        var generator = KeyPairGenerator.getInstance("EC");
        generator.initialize(new ECGenParameterSpec(curve));
        return generator.generateKeyPair();
    }

    private Path write(String name, String type, byte[] der) throws Exception {
        var body = Base64.getMimeEncoder(64, "\n".getBytes()).encodeToString(der);
        return Files.writeString(dir.resolve(name), "-----BEGIN " + type + "-----\n" + body + "\n-----END " + type + "-----\n");
    }
}
