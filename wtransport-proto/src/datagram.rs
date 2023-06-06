use crate::bytes::BufferReader;
use crate::bytes::BufferWriter;
use crate::bytes::BytesReader;
use crate::bytes::BytesWriter;
use crate::bytes::EndOfBuffer;
use crate::ids::InvalidQStreamId;
use crate::ids::QStreamId;

/// Error datagram read operation.
#[derive(Debug)]
pub enum DatagramReadError {
    /// Error when QUIC datagram is too short.
    TooShort,

    /// Error for invalid QStream ID.
    InvalidQStreamId,
}

/// An HTTP3 datagram.
pub struct Datagram<'a> {
    qstream_id: QStreamId,
    payload: &'a [u8],
}

impl<'a> Datagram<'a> {
    /// Creates a new [`Datagram`] with a given payload.
    #[inline(always)]
    pub fn new(qstream_id: QStreamId, payload: &'a [u8]) -> Self {
        Self {
            qstream_id,
            payload,
        }
    }

    /// Reads [`Datagram`] from a QUIC datagram.
    pub fn read(quic_datagram: &'a [u8]) -> Result<Self, DatagramReadError> {
        let mut buffer_reader = BufferReader::new(quic_datagram);

        let varint = buffer_reader
            .get_varint()
            .ok_or(DatagramReadError::TooShort)?;

        let qstream_id = QStreamId::try_from_varint(varint)
            .map_err(|InvalidQStreamId| DatagramReadError::InvalidQStreamId)?;

        let payload = buffer_reader.buffer_remaining();

        Ok(Self {
            qstream_id,
            payload,
        })
    }

    /// Writes a [`Datagram`] as QUIC datagram into `buffer`..
    ///
    /// It returns [`Err`] if the `buffer` does not have enough capacity.
    /// See [`Self::write_size`].
    ///
    /// In case of [`Err`], `buffer` is not written.
    pub fn write(&self, buffer: &mut [u8]) -> Result<(), EndOfBuffer> {
        if buffer.len() < self.write_size() {
            return Err(EndOfBuffer);
        }

        let mut buffer_writer = BufferWriter::new(buffer);

        buffer_writer
            .put_varint(self.qstream_id.into_varint())
            .expect("Buffer has capacity");

        buffer_writer
            .put_bytes(self.payload)
            .expect("Buffer has capacity");

        Ok(())
    }

    /// Returns the needed capacity to write this [`Datagram`] into a buffer.
    // TODO(bfesta): you should implement this logic-method for `Frame` and `StreamHeader` as well!
    #[inline(always)]
    pub fn write_size(&self) -> usize {
        self.qstream_id.into_varint().size() + self.payload.len()
    }

    /// Returns the associated [`QStreamId`].
    #[inline(always)]
    pub fn qstream_id(&self) -> QStreamId {
        self.qstream_id
    }

    /// Returns the payload.
    #[inline(always)]
    pub fn payload(&self) -> &[u8] {
        self.payload
    }
}