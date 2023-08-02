

use std::io;
use std::str::FromStr;

use bitcoin::Amount;
use bitcoin::secp256k1::SecretKey;
use elements::AssetId;
use elements::opcodes::all::*;
use elements::script::Builder;

/// Create a burn output, this is critical as this will be encoded
/// into the covenant script, so it needs to be deterministic.
pub fn burn_output(amount: Amount, asset: AssetId) -> elements::TxOut {
	elements::TxOut {
		value: elements::confidential::Value::Explicit(amount.to_sat()),
		asset: elements::confidential::Asset::Explicit(asset),
		nonce: elements::confidential::Nonce::Null,
		script_pubkey: Builder::new().push_opcode(OP_RETURN).into_script(),
		witness: elements::TxOutWitness::default(),
	}
}

pub trait BuilderExt: Into<Builder> + From<Builder> {
	/// Check that the top stack item is of the required size.
	fn check_stack_item_size(self, size: i64) -> Self {
		self.into()
			.push_opcode(OP_SIZE)
			.push_int(size)
			.push_opcode(OP_EQUALVERIFY)
			.into()
	}

}

impl BuilderExt for Builder {}

pub trait BitcoinEncodableExt: bitcoin::consensus::encode::Encodable {
	fn encoded_len(&self) -> usize {
		let mut counter = ByteCountSink::default();
		self.consensus_encode(&mut counter).unwrap();
		counter.count
	}
}

impl<T: bitcoin::consensus::encode::Encodable + ?Sized> BitcoinEncodableExt for T {}

pub trait ElementsEncodableExt: elements::encode::Encodable {
	fn encoded_len(&self) -> usize {
		let mut counter = ByteCountSink::default();
		self.consensus_encode(&mut counter).unwrap();
		counter.count
	}
}

impl<T: elements::encode::Encodable + ?Sized> ElementsEncodableExt for T {}

#[derive(Default)]
struct ByteCountSink {
	count: usize,
}

impl io::Write for ByteCountSink {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
		let len = buf.len();
		self.count += len;
		Ok(len)
	}
    fn flush(&mut self) -> Result<(), io::Error> {
		Ok(())
	}
}

pub trait ReadExt: io::Read {
	fn take_bytes(&mut self, n: usize) -> Result<Vec<u8>, io::Error> {
		let mut buf = vec![0; n];
		self.read_exact(&mut buf)?;
		Ok(buf)
	}
}
impl<T: AsRef<[u8]>> ReadExt for io::Cursor<T> {}

/// Parse a secret key from a string.
/// Supports both WIF format and hexadecimal.
pub fn parse_secret_key(s: &str) -> Result<SecretKey, String> {
	if let Ok(k) = bitcoin::PrivateKey::from_str(&s) {
		Ok(k.inner)
	} else {
		Ok(SecretKey::from_str(&s).map_err(|_| "invalid secret key")?)
	}
}
