use crate::aps::*;
use crate::parsers::error::{ParseError, ParseResult};
use nom::{
    bytes::complete::take,
    number::complete::{le_u16, le_u8},
    IResult,
};

/// Parse APS frame
pub fn parse_aps_frame(input: &[u8]) -> ParseResult<ApsFrame> {
    match parse_aps_frame_nom(input) {
        Ok((_, frame)) => Ok(frame),
        Err(nom::Err::Incomplete(_)) => Err(ParseError::Incomplete { needed: 0 }),
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            Err(ParseError::ParseFailed(format!("APS parse failed: {:?}", e)))
        }
    }
}

fn parse_aps_frame_nom(input: &[u8]) -> IResult<&[u8], ApsFrame> {
    if input.is_empty() {
        return Err(nom::Err::Incomplete(nom::Needed::new(1)));
    }
    
    let (input, frame_control_byte) = le_u8(input)?;
    let frame_control = parse_aps_frame_control(frame_control_byte);
    
    // Destination endpoint (if not group addressing)
    let (input, dest_endpoint) = if frame_control.delivery_mode != DeliveryMode::Group {
        let (input, ep) = le_u8(input)?;
        (input, Some(ep))
    } else {
        (input, None)
    };
    
    // Group address (if group addressing) - not implemented yet
    let input = if frame_control.delivery_mode == DeliveryMode::Group {
        let (input, _group_addr) = le_u16(input)?;
        input
    } else {
        input
    };
    
    // Cluster ID (for data frames)
    let (input, cluster_id) = if frame_control.frame_type == ApsFrameType::Data {
        let (input, cluster) = le_u16(input)?;
        (input, Some(cluster))
    } else {
        (input, None)
    };
    
    // Profile ID (for data frames)
    let (input, profile_id) = if frame_control.frame_type == ApsFrameType::Data {
        let (input, profile) = le_u16(input)?;
        (input, Some(profile))
    } else {
        (input, None)
    };
    
    // Source endpoint
    let (input, src_endpoint) = if frame_control.frame_type == ApsFrameType::Data 
        || frame_control.frame_type == ApsFrameType::Command {
        let (input, ep) = le_u8(input)?;
        (input, Some(ep))
    } else {
        (input, None)
    };
    
    // APS counter
    let (input, counter) = le_u8(input)?;
    
    // Extended header (if present)
    let (input, extended_header) = if frame_control.extended_header_present {
        // Extended header format is variable, just grab it as bytes for now
        // TODO: Proper extended header parsing
        let (input, ext_len) = le_u8(input)?;
        let (input, ext_data) = take(ext_len)(input)?;
        (input, Some(ext_data.to_vec()))
    } else {
        (input, None)
    };
    
    // Remaining bytes are payload
    let payload = input.to_vec();
    
    Ok((
        &[],
        ApsFrame {
            frame_control,
            dest_endpoint,
            cluster_id,
            profile_id,
            src_endpoint,
            counter,
            extended_header,
            payload,
        },
    ))
}

fn parse_aps_frame_control(fc: u8) -> ApsFrameControl {
    let frame_type = ApsFrameType::from(fc & 0x03);
    let delivery_mode = DeliveryMode::from((fc >> 2) & 0x03);
    let ack_format = (fc & 0x10) != 0;
    let security = (fc & 0x20) != 0;
    let ack_request = (fc & 0x40) != 0;
    let extended_header_present = (fc & 0x80) != 0;
    
    ApsFrameControl {
        frame_type,
        delivery_mode,
        ack_format,
        security,
        ack_request,
        extended_header_present,
    }
}