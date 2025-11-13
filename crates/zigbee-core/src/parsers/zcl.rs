use crate::zcl::*;
use crate::parsers::error::{ParseError, ParseResult};
use nom::{
    number::complete::{le_u16, le_u8},
    IResult,
};

/// Parse ZCL frame
pub fn parse_zcl_frame(input: &[u8]) -> ParseResult<ZclFrame> {
    match parse_zcl_frame_nom(input) {
        Ok((_, frame)) => Ok(frame),
        Err(nom::Err::Incomplete(_)) => Err(ParseError::Incomplete { needed: 0 }),
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            Err(ParseError::ParseFailed(format!("ZCL parse failed: {:?}", e)))
        }
    }
}

fn parse_zcl_frame_nom(input: &[u8]) -> IResult<&[u8], ZclFrame> {
    if input.is_empty() {
        return Err(nom::Err::Incomplete(nom::Needed::new(1)));
    }
    
    let (input, frame_control_byte) = le_u8(input)?;
    let frame_control = parse_zcl_frame_control(frame_control_byte);
    
    // Manufacturer code (if manufacturer specific)
    let (input, manufacturer_code) = if frame_control.manufacturer_specific {
        let (input, code) = le_u16(input)?;
        (input, Some(code))
    } else {
        (input, None)
    };
    
    // Transaction sequence number
    let (input, transaction_sequence) = le_u8(input)?;
    
    // Command identifier
    let (input, command) = le_u8(input)?;
    
    // Remaining bytes are payload
    let payload = input.to_vec();
    
    Ok((
        &[],
        ZclFrame {
            frame_control,
            manufacturer_code,
            transaction_sequence,
            command,
            payload,
        },
    ))
}

fn parse_zcl_frame_control(fc: u8) -> ZclFrameControl {
    let frame_type = if (fc & 0x01) == 0 {
        ZclFrameType::Global
    } else {
        ZclFrameType::ClusterSpecific
    };
    
    let manufacturer_specific = (fc & 0x04) != 0;
    
    let direction = if (fc & 0x08) == 0 {
        ZclDirection::ClientToServer
    } else {
        ZclDirection::ServerToClient
    };
    
    let disable_default_response = (fc & 0x10) != 0;
    
    ZclFrameControl {
        frame_type,
        manufacturer_specific,
        direction,
        disable_default_response,
    }
}