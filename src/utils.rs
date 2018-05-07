use byteorder::{ByteOrder, LittleEndian as LE, WriteBytesExt};

use super::{BestTimes, ElmaError, Time, TimeEntry};

/// Parse top10 lists and return a vector of `TimeEntry`s
pub fn parse_top10(top10: &[u8]) -> Result<Vec<TimeEntry>, ElmaError> {
    let mut list: Vec<TimeEntry> = vec![];
    let times = LE::read_i32(&top10[0..4]);
    for n in 0..times as usize {
        let time_offset = 4 + n * 4;
        let time_end = time_offset + 4;
        let name_1_offset = 44 + n * 15;
        let name_1_end = name_1_offset + 15;
        let name_2_offset = 194 + n * 15;
        let name_2_end = name_2_offset + 15;

        let name_1 = &top10[name_1_offset..name_1_end];
        let name_2 = &top10[name_2_offset..name_2_end];
        let time = &top10[time_offset..time_end];
        list.push(TimeEntry {
            time: Time(LE::read_i32(time)),
            names: (trim_string(name_1)?, trim_string(name_2)?),
        });
    }
    Ok(list)
}

/// Write `best times` data as bytes.
pub fn write_top10(best_times: &BestTimes) -> Result<Vec<u8>, ElmaError> {
    let mut top10_bytes: Vec<u8> = vec![];

    // Single-player times.
    let single_times = best_times.single.len();
    top10_bytes.write_i32::<LE>(if 10 < single_times { 10 } else { single_times } as i32)?;
    let mut times = [0_i32; 10];
    let mut names_1 = vec![];
    let mut names_2 = vec![];
    for (n, entry) in best_times.single.iter().enumerate() {
        if n < 10 {
            times[n] = entry.time.into();
            names_1.extend_from_slice(&string_null_pad(&entry.names.0, 15)?);
            names_2.extend_from_slice(&string_null_pad(&entry.names.1, 15)?);
        }
    }
    // Pad with null bytes if less than 10 entries.
    if single_times < 10 {
        for _ in 0..10 - single_times {
            names_1.extend_from_slice(&[0u8; 15]);
            names_2.extend_from_slice(&[0u8; 15]);
        }
    }

    for time in &times {
        top10_bytes.write_i32::<LE>(*time)?;
    }

    top10_bytes.extend_from_slice(&names_1);
    top10_bytes.extend_from_slice(&names_2);

    // Multi-player times.
    let multi_times = best_times.multi.len();
    top10_bytes.write_i32::<LE>(if 10 < multi_times { 10 } else { multi_times } as i32)?;
    let mut times = [0_i32; 10];
    let mut names_1 = vec![];
    let mut names_2 = vec![];
    for (n, entry) in best_times.multi.iter().enumerate() {
        if n < 10 {
            times[n] = entry.time.into();
            names_1.extend_from_slice(&string_null_pad(&entry.names.0, 15)?);
            names_2.extend_from_slice(&string_null_pad(&entry.names.1, 15)?);
        }
    }
    // Pad with null bytes if less than 10 entries.
    if multi_times < 10 {
        for _ in 0..10 - multi_times {
            names_1.extend_from_slice(&[0u8; 15]);
            names_2.extend_from_slice(&[0u8; 15]);
        }
    }

    for time in &times {
        top10_bytes.write_i32::<LE>(*time)?;
    }

    top10_bytes.extend_from_slice(&names_1);
    top10_bytes.extend_from_slice(&names_2);

    Ok(top10_bytes)
}

/// Trims trailing bytes after and including null byte.
///
/// # Examples
/// As all strings in Elma files are C-strings with padded null-bytes, you can use this function
/// to remove null-bytes and any potential garbage data follwing it and return a String.
///
/// ```
/// let cstring: [u8; 10] = [0x45, 0x6C, 0x6D, 0x61, 0x00, 0x00, 0x00, 0x7E, 0x7E, 0x7E];
/// let trimmed = elma::utils::trim_string(&cstring).unwrap();
/// assert_eq!(trimmed, "Elma");
/// ```
pub fn trim_string(data: &[u8]) -> Result<String, ElmaError> {
    let bytes: Vec<u8> = data.into_iter().take_while(|&&d| d != 0).cloned().collect();

    let trimmed = String::from_utf8(bytes)?;
    Ok(trimmed)
}

/// Pads a string with null bytes.
///
/// # Examples
/// When converting strings to bytes for use in an Elma file, you need to pad it to a certain
/// length depending on the field. This function creates a new zero-filled vector to `pad` size,
/// then fills in the string.
///
/// ```
/// let string = String::from("Elma");
/// let padded = elma::utils::string_null_pad(&string, 10).unwrap();
/// assert_eq!(&padded, &[0x45, 0x6C, 0x6D, 0x61, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
/// ```
pub fn string_null_pad(name: &str, pad: usize) -> Result<Vec<u8>, ElmaError> {
    let name = name.as_bytes();

    // first check if string is ASCII
    if !name.is_ascii() {
        return Err(ElmaError::NonASCII);
    }
    // padding shorter than string
    if name.len() > pad {
        return Err(ElmaError::PaddingTooShort(
            (pad as isize - name.len() as isize) as isize,
        ));
    }

    let mut bytes = vec![0u8; pad];
    for (n, char) in name.iter().enumerate() {
        bytes[n] = *char;
    }
    Ok(bytes)
}