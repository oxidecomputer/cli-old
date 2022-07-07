use std::io::BufRead;

use anyhow::Result;
use parse_display::{Display, FromStr};
use ring::{
    rand,
    signature::{EcdsaKeyPair, Ed25519KeyPair, KeyPair, ECDSA_P256_SHA256_FIXED_SIGNING},
};

/// Retrieve the public SSH keys for a specific github user.
pub async fn get_github_ssh_keys(gh_handle: &str) -> Result<Vec<sshkeys::PublicKey>> {
    let resp = reqwest::get(&format!("https://github.com/{}.keys", gh_handle)).await?;
    let body = resp.bytes().await?;

    let reader = std::io::BufReader::new(body.as_ref());
    let lines: Vec<_> = reader.lines().collect();

    let mut keys: Vec<sshkeys::PublicKey> = Vec::new();
    for l in lines {
        let line = l?;
        // Parse the key.
        let key = sshkeys::PublicKey::from_string(&line)?;

        // Add the key to the list.
        keys.push(key);
    }

    Ok(keys)
}

#[derive(Debug, Clone, PartialEq, Eq, FromStr, Display)]
#[display(style = "kebab-case")]
pub enum SSHKeyAlgorithm {
    Rsa,
    Ed25519,
    Ecdsa,
}

impl Default for SSHKeyAlgorithm {
    fn default() -> SSHKeyAlgorithm {
        SSHKeyAlgorithm::Ed25519
    }
}

/// A public and private keypair.
#[derive(Debug)]
pub enum SSHKeyPair {
    // TODO: do RSA.
    //Rsa(PKey<K>),
    Ecdsa(EcdsaKeyPair),
    Ed25519(Ed25519KeyPair),
}

#[allow(dead_code)]
impl SSHKeyPair {
    /// Generate a new ssh key pair.
    pub fn generate(algorithm: &SSHKeyAlgorithm) -> Result<Self> {
        let key = match algorithm {
            SSHKeyAlgorithm::Rsa => {
                todo!("generate an RSA key")
            }
            SSHKeyAlgorithm::Ecdsa => {
                let rng = rand::SystemRandom::new();
                let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng)
                    .map_err(|e| anyhow::anyhow!("{}", e))
                    .map(|pkcs8_bytes| pkcs8_bytes.as_ref().to_vec())?;
                let k = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref())
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                SSHKeyPair::Ecdsa(k)
            }
            SSHKeyAlgorithm::Ed25519 => {
                let rng = rand::SystemRandom::new();
                let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng)
                    .map_err(|e| anyhow::anyhow!("{}", e))
                    .map(|pkcs8_bytes| pkcs8_bytes.as_ref().to_vec())?;
                let k = Ed25519KeyPair::from_pkcs8(&pkcs8).map_err(|e| anyhow::anyhow!("{}", e))?;
                SSHKeyPair::Ed25519(k)
            }
        };

        Ok(key)
    }

    pub fn public_key(&self) -> Result<sshkeys::PublicKey> {
        let mut w = sshkeys::Writer::new();

        let (t, pk) = match self {
            SSHKeyPair::Ecdsa(ec_key) => {
                let mut bytes: Vec<u8> = ec_key.public_key().as_ref().to_vec();
                bytes.remove(0);

                let t = "ecdsa-sha2-nistp256";

                w.write_string(t);
                w.write_string("nistp256");

                (t, bytes)
            }
            SSHKeyPair::Ed25519(ed_key) => {
                let t = "ssh-ed25519";
                w.write_string(t);

                (t, ed_key.public_key().as_ref().to_vec())
            }
        };

        w.write_bytes(&pk);

        let bytes = w.into_bytes();

        let pk_str = format!("{} {}", t, data_encoding::BASE64.encode(&bytes));

        let key = sshkeys::PublicKey::from_string(&pk_str)?;

        Ok(key)
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn test_get_github_ssh_keys() {
        let result = super::get_github_ssh_keys("jessfraz").await;
        assert_eq!(result.is_ok(), true);

        let keys = result.unwrap();

        assert_eq!(!keys.is_empty(), true);
    }

    #[test]
    fn test_ssh_key_generate_ed25519() {
        let result = super::SSHKeyPair::generate(&super::SSHKeyAlgorithm::Ed25519);
        assert_eq!(result.is_ok(), true);

        let key = result.unwrap();
        let pub_key = key.public_key().unwrap();

        assert_eq!(pub_key.key_type.name, "ssh-ed25519");
        assert_eq!(!pub_key.fingerprint().to_string().is_empty(), true);
    }

    #[test]
    fn test_ssh_key_generate_ecdsa() {
        let result = super::SSHKeyPair::generate(&super::SSHKeyAlgorithm::Ecdsa);
        assert_eq!(result.is_ok(), true);

        let key = result.unwrap();
        let pub_key = key.public_key().unwrap();

        assert_eq!(pub_key.key_type.name, "ecdsa-sha2-nistp256");
        assert_eq!(!pub_key.fingerprint().to_string().is_empty(), true);
    }
}
