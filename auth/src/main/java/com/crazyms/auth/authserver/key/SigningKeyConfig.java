package com.crazyms.auth.authserver.key;

import com.crazyms.auth.config.AuthProperties;
import com.nimbusds.jose.JOSEException;
import com.nimbusds.jose.JWSAlgorithm;
import com.nimbusds.jose.jwk.Curve;
import com.nimbusds.jose.jwk.ECKey;
import com.nimbusds.jose.jwk.JWKSet;
import com.nimbusds.jose.jwk.KeyUse;
import com.nimbusds.jose.jwk.source.ImmutableJWKSet;
import com.nimbusds.jose.jwk.source.JWKSource;
import com.nimbusds.jose.proc.SecurityContext;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.GeneralSecurityException;
import java.security.KeyFactory;
import java.security.Signature;
import java.security.interfaces.ECPrivateKey;
import java.security.interfaces.ECPublicKey;
import java.security.spec.PKCS8EncodedKeySpec;
import java.security.spec.X509EncodedKeySpec;
import java.util.Base64;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

/**
 * The only place that knows where signing keys come from. Every replica must share them, so they are never
 * generated here; swapping PEM files for a KMS or Vault means replacing this class and nothing else.
 */
@Configuration(proxyBeanMethods = false)
class SigningKeyConfig {

    @Bean
    JWKSource<SecurityContext> jwkSource(AuthProperties props)
            throws IOException, GeneralSecurityException, JOSEException {
        return new ImmutableJWKSet<>(new JWKSet(loadSigningKey(props)));
    }

    private static ECKey loadSigningKey(AuthProperties props)
            throws IOException, GeneralSecurityException, JOSEException {
        var kf = KeyFactory.getInstance("EC");
        var priv = (ECPrivateKey) kf.generatePrivate(
                new PKCS8EncodedKeySpec(pemBody(props.privateKeyPath(), "PRIVATE KEY")));
        var pub = (ECPublicKey) kf.generatePublic(
                new X509EncodedKeySpec(pemBody(props.publicKeyPath(), "PUBLIC KEY")));

        if (!Curve.P_256.equals(Curve.forECParameterSpec(pub.getParams()))) {
            throw new IllegalStateException("signing key must be on curve P-256");
        }
        requireMatchingPair(priv, pub);

        return new ECKey.Builder(Curve.P_256, pub)
                .privateKey(priv)
                .keyUse(KeyUse.SIGNATURE)
                .algorithm(JWSAlgorithm.ES256)
                .keyIDFromThumbprint()
                .build();
    }

    private static byte[] pemBody(Path path, String type) throws IOException {
        var pem = Files.readString(path, StandardCharsets.US_ASCII);
        var begin = "-----BEGIN " + type + "-----";
        var end = "-----END " + type + "-----";
        int from = pem.indexOf(begin);
        int to = pem.indexOf(end);
        if (from < 0 || to < from) {
            throw new IllegalStateException(path + " is not a PEM " + type);
        }
        return Base64.getMimeDecoder().decode(pem.substring(from + begin.length(), to));
    }

    // A mismatched pair would issue tokens that nobody can verify; fail at startup instead.
    private static void requireMatchingPair(ECPrivateKey priv, ECPublicKey pub) throws GeneralSecurityException {
        byte[] probe = "key-pair-check".getBytes(StandardCharsets.US_ASCII);
        var signer = Signature.getInstance("SHA256withECDSA");
        signer.initSign(priv);
        signer.update(probe);
        byte[] sig = signer.sign();
        var verifier = Signature.getInstance("SHA256withECDSA");
        verifier.initVerify(pub);
        verifier.update(probe);
        if (!verifier.verify(sig)) {
            throw new IllegalStateException("private and public signing keys do not match");
        }
    }
}
