use bytes::Bytes;
use futures_util::TryStreamExt;
use std::collections::TryReserveError;

use cid::Cid;
use futures_util::TryStream;
use multihash_codetable::Code;
use multihash_codetable::MultihashDigest;
use serde_ipld_dagcbor::EncodeError;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek};

pub fn generate_cid<B>(data: B) -> Result<Cid, EncodeError<TryReserveError>>
where
    B: AsRef<[u8]>,
{
    let mh = Code::Sha2_256.digest(data.as_ref());
    let cid = Cid::new_v1(multicodec::Codec::DagCbor.code(), mh);

    Ok(cid)
}

pub fn generate_cid_from_serialized<T: serde::Serialize>(
    data: T,
) -> Result<Cid, EncodeError<TryReserveError>> {
    let serialized = serde_ipld_dagcbor::to_vec(&data)?;
    generate_cid(serialized)
}

pub async fn generate_cid_from_stream<S: TryStream<Ok = Bytes> + Unpin>(
    stream: S,
) -> Result<Cid, EncodeError<TryReserveError>>
where
    S::Error: Into<EncodeError<TryReserveError>>,
{
    let mut buf = Vec::new();
    let _ = stream
        .try_for_each(|chunk| {
            buf.extend_from_slice(&chunk);
            async { Ok(()) }
        })
        .await;

    let mh = Code::Sha2_256.digest(&buf);
    let cid = Cid::new_v1(multicodec::Codec::DagCbor.code(), mh);

    Ok(cid)
}

pub async fn generate_cid_from_asyncreader<R>(
    reader: R,
) -> Result<Cid, EncodeError<TryReserveError>>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    let mut buf = Vec::new();
    reader
        .take(1024 * 1024)
        .read_to_end(&mut buf)
        .await
        .map_err(EncodeError::Write)
        .unwrap();

    let mh = Code::Sha2_256.digest(&buf);
    let cid = Cid::new_v1(multicodec::Codec::DagCbor.code(), mh);

    Ok(cid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Cursor;
    use std::str::FromStr;

    #[test]
    fn test_generate_cid() {
        let data = json!({
            "hello": "world",
        });

        let cid = generate_cid(data.to_string()).unwrap();

        assert_eq!(
            cid,
            Cid::from_str("bafyreietui4xdkiu4xvmx4fi2jivjtndbhb4drzpxomrjvd4mdz4w2avra").unwrap(),
        );
        assert_eq!(cid.codec(), multicodec::Codec::DagCbor.code());
    }

    #[tokio::test]
    async fn test_generate_cid_from_asyncreader() {
        // Define some sample data to read
        let data = b"Sample data to generate CID";

        // Create a cursor over the data, which implements AsyncRead + AsyncSeek
        let cursor = Cursor::new(data);

        // Call the function with the cursor
        let cid = generate_cid_from_asyncreader(cursor).await;
        assert!(cid.is_ok());
        let cid = cid.unwrap();

        // Verify that the CID is generated correctly
        // For a real test, you might compare the cid with a known value
        assert_eq!(cid.version(), cid::Version::V1);
        assert_eq!(cid.codec(), multicodec::Codec::DagCbor.code());

        // For demonstration: hash the data using the same logic to get the expected hash
        let expected_mh = multihash_codetable::Code::Sha2_256.digest(data);

        // Compare multihashes
        assert_eq!(cid.hash(), &expected_mh);
    }
}
