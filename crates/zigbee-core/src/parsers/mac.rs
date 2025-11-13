use crate::ieee802154::*;
use crate::parsers::error::{ParseError, ParseResult};
use nom::{
    bytes::complete::take,
    number::complete::{le_u16, le_u8},
    IResult,
};

/// Parse a complete MAC frame from raw bytes
pub fn parse_mac_frame(input: &[u8]) -> ParseResult<MacFrame> {
    match parse_mac_frame_nom(input) {
        Ok((_, frame)) => Ok(frame),
        Err(nom::Err::Incomplete(_)) => Err(ParseError::Incomplete { needed: 0 }),
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            Err(ParseError::ParseFailed(format!("MAC parse failed: {:?}", e)))
        }
    }
}

fn parse_mac_frame_nom(input: &[u8]) -> IResult<&[u8], MacFrame> {
    if input.len() < 5 {
        return Err(nom::Err::Incomplete(nom::Needed::new(5 - input.len())));
    }
    
    let (input, frame_control_bytes) = le_u16(input)?;
    let frame_control = parse_frame_control(frame_control_bytes);
    
    let (input, sequence) = le_u8(input)?;
    
    // Parse destination PAN and address
       let (input, (dest_pan, dest_addr)) = parse_address_fields(
        input,
        frame_control.dest_addr_mode,
        true, // destination gets PAN ID unless compressed
       )?;
    
    // Parse source PAN and address
    let pan_id_compression = frame_control.pan_id_compression;
    let (input, (src_pan, src_addr)) = parse_address_fields(
        input,
        frame_control.src_addr_mode,
        !pan_id_compression, // source only gets PAN if not compressed
    )?;
    
    // Security header (if present)
    let (input, security) = if frame_control.security_enabled {
        // TODO: Parse security header
        return Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        )));
    } else {
        (input, None)
    };
    
    // Payload is everything except last 2 bytes (FCS)
    if input.len() < 2 {
        return Err(nom::Err::Incomplete(nom::Needed::new(2)));
    }
    
    let payload_len = input.len() - 2;
    let (input, payload) = take(payload_len)(input)?;
    let (input, fcs) = le_u16(input)?;
    
    Ok((
        input,
        MacFrame {
            frame_control,
            sequence,
            dest_pan,
            dest_addr,
            src_pan,
            src_addr,
            security,
            payload: payload.to_vec(),
            fcs,
        },
    ))
}

fn parse_frame_control(fc: u16) -> FrameControl {
    let frame_type = FrameType::from((fc & 0x07) as u8);
    let security_enabled = (fc & 0x08) != 0;
    let frame_pending = (fc & 0x10) != 0;
    let ack_request = (fc & 0x20) != 0;
    let pan_id_compression = (fc & 0x40) != 0;
    let dest_addr_mode = AddressingMode::from(((fc >> 10) & 0x03) as u8);
    let frame_version = ((fc >> 12) & 0x03) as u8;
    let src_addr_mode = AddressingMode::from(((fc >> 14) & 0x03) as u8);
    
    FrameControl {
        frame_type,
        security_enabled,
        frame_pending,
        ack_request,
        pan_id_compression,
        dest_addr_mode,
        frame_version,
        src_addr_mode,
    }
}

fn parse_address_fields(
    input: &[u8],
    addr_mode: AddressingMode,
    include_pan: bool,
) -> IResult<&[u8], (Option<u16>, MacAddress)> {
    let (input, pan_id) = if include_pan && addr_mode != AddressingMode::None {
        let (input, pan) = le_u16(input)?;
        (input, Some(pan))
    } else {
        (input, None)
    };
    
    let (input, address) = match addr_mode {
        AddressingMode::None => (input, MacAddress::None),
        AddressingMode::Short => {
            let (input, addr) = le_u16(input)?;
            (input, MacAddress::Short(addr))
        }
        AddressingMode::Extended => {
            let (input, addr_bytes) = take(8usize)(input)?;
            let mut addr = [0u8; 8];
            addr.copy_from_slice(addr_bytes);
            (input, MacAddress::Extended(addr))
        }
        AddressingMode::Reserved => (input, MacAddress::None),
    };
    
    Ok((input, (pan_id, address)))
}