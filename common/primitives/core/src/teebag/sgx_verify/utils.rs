fn safe_indexing_one(data: &[u8], idx: usize) -> Result<usize, &'static str> {
    let elt = data.get(idx).ok_or("Index out of bounds")?;
    Ok(*elt as usize)
}

pub fn length_from_raw_data(data: &[u8], offset: &mut usize) -> Result<usize, &'static str> {
    let mut len = safe_indexing_one(data, *offset)?;
    if len > 0x80 {
        len = (safe_indexing_one(data, *offset + 1)?) * 0x100
            + (safe_indexing_one(data, *offset + 2)?);
        *offset += 2;
    }
    Ok(len)
}

#[cfg(test)]
mod test {
    use super::*;
    use frame_support::assert_err;

    #[test]
    fn index_equal_length_returns_err() {
        // It was discovered a panic occurs if `index == data.len()` due to out of bound
        // indexing. Here the fix is tested.
        //
        // For context see: https://github.com/integritee-network/pallet-teerex/issues/34
        let data: [u8; 7] = [0, 1, 2, 3, 4, 5, 6];
        assert_err!(safe_indexing_one(&data, data.len()), "Index out of bounds");
    }

    #[test]
    fn safe_indexing_works() {
        let data: [u8; 7] = [0, 1, 2, 3, 4, 5, 6];
        assert_eq!(safe_indexing_one(&data, 0), Ok(0));
        assert_eq!(safe_indexing_one(&data, 3), Ok(3));
        assert!(safe_indexing_one(&data, 10).is_err());
    }
}
