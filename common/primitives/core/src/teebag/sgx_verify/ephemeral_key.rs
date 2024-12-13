use super::{utils::length_from_raw_data, CertDer};
use sp_std::convert::TryFrom;

#[allow(dead_code)]
pub struct EphemeralKey<'a>(&'a [u8]);

pub const PRIME256V1_OID: &[u8; 10] = &[0x06, 0x08, 0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x03, 0x01, 0x07];
impl<'a> TryFrom<CertDer<'a>> for EphemeralKey<'a> {
    type Error = &'static str;

    fn try_from(value: CertDer<'a>) -> Result<Self, Self::Error> {
        let cert_der = value.0;

        let mut offset = cert_der
            .windows(PRIME256V1_OID.len())
            .position(|window| window == PRIME256V1_OID)
            .ok_or("Certificate does not contain 'PRIME256V1_OID'")?;

        offset += PRIME256V1_OID.len() + 1; // OID length + TAG (0x03)

        // Obtain Public Key length
        let len = length_from_raw_data(cert_der, &mut offset)?;

        // Obtain Public Key
        offset += 1;
        let pub_k = cert_der
            .get(offset + 2..offset + len)
            .ok_or("Index out of bounds")?; // skip "00 04"

        #[cfg(test)]
        println!("verifyRA ephemeral public key: {:x?}", pub_k);
        Ok(EphemeralKey(pub_k))
    }
}
