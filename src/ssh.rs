use std::io::BufRead;

use anyhow::Result;
use data_encoding::BASE64;
use parse_display::{Display, FromStr};
use ring::{
    rand,
    signature::{EcdsaKeyPair, Ed25519KeyPair, KeyPair, ECDSA_P256_SHA256_FIXED_SIGNING},
};

/// Retrieve the public ssh keys for a specific github user.
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
    Rsa4098,
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

impl SSHKeyPair {
    /// Generate a new ssh key pair.
    pub fn generate(algorithm: &SSHKeyAlgorithm) -> Result<Self> {
        let key = match algorithm {
            SSHKeyAlgorithm::Rsa4098 => {
                todo!()
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

        println!("{:?}", key);

        Ok(key)
    }

    pub fn public_key(&self) -> Result<sshkeys::PublicKey> {
        let (t, pk) = match self {
            SSHKeyPair::Ecdsa(ec_key) => {
                let mut bytes: Vec<u8> = ec_key.public_key().as_ref().to_vec();
                bytes.remove(0);
                ("ecdsa-sha2-nistp256", bytes)
            }
            SSHKeyPair::Ed25519(ed_key) => ("ssh-ed25519", ed_key.public_key().as_ref().to_vec()),
        };

        // Format the public key.
        let pk_str = format!("{} {}", t, BASE64.encode(&pk));

        println!("{}", pk_str);

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

        assert_eq!(keys.len() > 0, true);
    }

    #[test]
    fn test_ssh_key_generate_ed25519() {
        let result = super::SSHKeyPair::generate(&super::SSHKeyAlgorithm::Ed25519);
        assert_eq!(result.is_ok(), true);

        let key = result.unwrap();
        let pub_key = key.public_key().unwrap();

        assert_eq!(pub_key.key_type.name, "thing");
        assert_eq!(pub_key.fingerprint().to_string(), "thing");
    }

    #[test]
    fn test_ssh_key_generate_ecdsa() {
        let result = super::SSHKeyPair::generate(&super::SSHKeyAlgorithm::Ecdsa);
        assert_eq!(result.is_ok(), true);

        let key = result.unwrap();
        let pub_key = key.public_key().unwrap();

        assert_eq!(pub_key.key_type.name, "thing");
        assert_eq!(pub_key.fingerprint().to_string(), "thing");
    }
}
