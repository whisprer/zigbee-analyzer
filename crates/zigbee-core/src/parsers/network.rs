use crate::network::*;
use crate::parsers::error::{ParseError, ParseResult};
use nom::{
    bytes::complete::take,
    number::complete::{le_u16, le_u8},
    IResult,
};

/// Parse Zigbee Network layer frame
pub fn parse_network_frame(input: &[u8]) -> ParseResult<NetworkFrame> {
    match parse_network_frame_nom(input) {
        Ok((_, frame)) => Ok(frame),
        Err(nom::Err::Incomplete(_)) => Err(ParseError::Incomplete { needed: 0 }),
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            Err(ParseError::ParseFailed(format!("NWK parse failed: {:?}", e)))
        }
    }
}

fn parse_network_frame_nom(input: &[u8]) -> IResult<&[u8], NetworkFrame> {
    if input.len() < 8 {
        return Err(nom::Err::Incomplete(nom::Needed::new(8 - input.len())));
    }
    
    let (input, frame_control_bytes) = le_u16(input)?;
    let frame_control = parse_nwk_frame_control(frame_control_bytes);
    
    let (input, dest_addr) = le_u16(input)?;
    let (input, src_addr) = le_u16(input)?;
    let (input, radius) = le_u8(input)?;
    let (input, sequence) = le_u8(input)?;
    
    // Optional destination IEEE address
    let (input, dest_ieee) = if frame_control.dest_ieee_present {
        let (input, ieee_bytes) = take(8usize)(input)?;
        let mut ieee = [0u8; 8];
        ieee.copy_from_slice(ieee_bytes);
        (input, Some(ieee))
    } else {
        (input, None)
    };
    
    // Optional source IEEE address
    let (input, src_ieee) = if frame_control.src_ieee_present {
        let (input, ieee_bytes) = take(8usize)(input)?;
        let mut ieee = [0u8; 8];
        ieee.copy_from_slice(ieee_bytes);
        (input, Some(ieee))
    } else {
        (input, None)
    };
    
    // Multicast control (if multicast flag set)
    let (input, multicast_control) = if frame_control.multicast {
        let (input, mc) = le_u8(input)?;
        (input, Some(mc))
    } else {
        (input, None)
    };
    
    // Source route subframe (if present) - skip for now
    let input = if frame_control.source_route {
        // TODO: Parse source route
        input
    } else {
        input
    };
    
    // Security frame (if present)
    let input = if frame_control.security {
        // TODO: Parse security
        return Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        )));
    } else {
        input
    };
    
    // Remaining bytes are payload
    let payload = input.to_vec();
    
    Ok((
        &[],
        NetworkFrame {
            frame_control,
            dest_addr,
            src_addr,
            radius,
            sequence,
            dest_ieee,
            src_ieee,
            multicast_control,
            payload,
        },
    ))
}

fn parse_nwk_frame_control(fc: u16) -> NwkFrameControl {
    let frame_type = NwkFrameType::from((fc & 0x03) as u8);
    let protocol_version = ((fc >> 2) & 0x0f) as u8;
    let discover_route = DiscoverRoute::from(((fc >> 6) & 0x03) as u8);
    let multicast = (fc & 0x0100) != 0;
    let security = (fc & 0x0200) != 0;
    let source_route = (fc & 0x0400) != 0;
    let dest_ieee_present = (fc & 0x0800) != 0;
    let src_ieee_present = (fc & 0x1000) != 0;
    
    NwkFrameControl {
        frame_type,
        protocol_version,
        discover_route,
        multicast,
        security,
        source_route,
        dest_ieee_present,
        src_ieee_present,
    }
}