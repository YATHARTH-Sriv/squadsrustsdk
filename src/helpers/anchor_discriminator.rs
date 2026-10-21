use sha2::Digest;

pub(crate) fn anchor_discriminator(value: &str) -> [u8; 8] {
    let hash = sha2::Sha256::digest(value.as_bytes());
    let mut discr = [0u8; 8];
    discr.copy_from_slice(&hash[0..8]);

    discr
}
