//! Excalidraw-compatible fractional ordering keys.
//!
//! Ported from Excalidraw's vendored `fractional-indexing` implementation,
//! which is CC0. Element arrays remain the convenient render-order cache, but
//! persisted `index` fields must sort in the same order for reconciliation.

pub const BASE_62_DIGITS: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub fn validate_order_key(key: &str) -> bool {
    validate_order_key_inner(key).is_ok()
}

pub fn generate_key_between(a: Option<&str>, b: Option<&str>) -> Option<String> {
    generate_key_between_inner(a, b).ok()
}

pub fn generate_n_keys_between(
    a: Option<&str>,
    b: Option<&str>,
    n: usize,
) -> Option<Vec<String>> {
    generate_n_keys_between_inner(a, b, n).ok()
}

fn midpoint(a: &str, b: Option<&str>) -> Result<String, ()> {
    let zero = digit(0)?;
    if b.is_some_and(|b| a >= b) || a.as_bytes().last() == Some(&zero) {
        return Err(());
    }
    if b.is_some_and(|b| b.as_bytes().last() == Some(&zero)) {
        return Err(());
    }

    if let Some(b) = b.filter(|b| !b.is_empty()) {
        let mut n = 0;
        let a_bytes = a.as_bytes();
        let b_bytes = b.as_bytes();
        while n < b_bytes.len() && a_bytes.get(n).copied().unwrap_or(zero) == b_bytes[n] {
            n += 1;
        }
        if n > 0 {
            let mut result = b[..n].to_owned();
            result.push_str(&midpoint(&a[n.min(a.len())..], Some(&b[n..]))?);
            return Ok(result);
        }
    }

    let digit_a = a
        .as_bytes()
        .first()
        .map_or(0, |value| digit_index(*value).unwrap_or(0));
    let digit_b = match b {
        Some(value) => value
            .as_bytes()
            .first()
            .and_then(|value| digit_index(*value))
            .unwrap_or(BASE_62_DIGITS.len()),
        None => BASE_62_DIGITS.len(),
    };

    if digit_b > digit_a + 1 {
        let mid = ((digit_a + digit_b) as f32 * 0.5).round() as usize;
        return Ok(char::from(digit(mid)?).to_string());
    }

    if let Some(b) = b.filter(|b| b.len() > 1) {
        return Ok(b[..1].to_owned());
    }

    let mut result = String::new();
    result.push(char::from(digit(digit_a)?));
    let tail = if a.is_empty() { "" } else { &a[1..] };
    result.push_str(&midpoint(tail, None)?);
    Ok(result)
}

fn validate_order_key_inner(key: &str) -> Result<(), ()> {
    if key.is_empty()
        || key == format!("A{}", "0".repeat(26))
        || !key.bytes().all(|value| digit_index(value).is_some())
    {
        return Err(());
    }
    let integer = get_integer_part(key)?;
    let fractional = &key[integer.len()..];
    if fractional.ends_with('0') {
        return Err(());
    }
    Ok(())
}

fn get_integer_length(head: u8) -> Result<usize, ()> {
    match head {
        b'a'..=b'z' => Ok(usize::from(head - b'a') + 2),
        b'A'..=b'Z' => Ok(usize::from(b'Z' - head) + 2),
        _ => Err(()),
    }
}

fn get_integer_part(key: &str) -> Result<&str, ()> {
    let head = *key.as_bytes().first().ok_or(())?;
    let length = get_integer_length(head)?;
    if length > key.len() {
        return Err(());
    }
    let integer = &key[..length];
    if integer.len() != get_integer_length(integer.as_bytes()[0])? {
        return Err(());
    }
    Ok(integer)
}

fn increment_integer(value: &str) -> Result<Option<String>, ()> {
    validate_integer(value)?;
    let mut bytes = value.as_bytes().to_vec();
    let head = bytes[0];
    let mut carry = true;
    for index in (1..bytes.len()).rev() {
        if !carry {
            break;
        }
        let next = digit_index(bytes[index]).ok_or(())? + 1;
        if next == BASE_62_DIGITS.len() {
            bytes[index] = digit(0)?;
        } else {
            bytes[index] = digit(next)?;
            carry = false;
        }
    }
    if carry {
        if head == b'Z' {
            return Ok(Some("a0".to_owned()));
        }
        if head == b'z' {
            return Ok(None);
        }
        let next_head = head + 1;
        bytes[0] = next_head;
        if next_head > b'a' {
            bytes.push(digit(0)?);
        } else {
            bytes.pop();
        }
    }
    String::from_utf8(bytes).map(Some).map_err(|_| ())
}

fn decrement_integer(value: &str) -> Result<Option<String>, ()> {
    validate_integer(value)?;
    let mut bytes = value.as_bytes().to_vec();
    let head = bytes[0];
    let mut borrow = true;
    for index in (1..bytes.len()).rev() {
        if !borrow {
            break;
        }
        let current = digit_index(bytes[index]).ok_or(())?;
        if current == 0 {
            bytes[index] = digit(BASE_62_DIGITS.len() - 1)?;
        } else {
            bytes[index] = digit(current - 1)?;
            borrow = false;
        }
    }
    if borrow {
        if head == b'a' {
            return Ok(Some("Zz".to_owned()));
        }
        if head == b'A' {
            return Ok(None);
        }
        let next_head = head - 1;
        bytes[0] = next_head;
        if next_head < b'Z' {
            bytes.push(digit(BASE_62_DIGITS.len() - 1)?);
        } else {
            bytes.pop();
        }
    }
    String::from_utf8(bytes).map(Some).map_err(|_| ())
}

fn validate_integer(value: &str) -> Result<(), ()> {
    let head = *value.as_bytes().first().ok_or(())?;
    (value.len() == get_integer_length(head)?)
        .then_some(())
        .ok_or(())
}

fn generate_key_between_inner(a: Option<&str>, b: Option<&str>) -> Result<String, ()> {
    if a.is_some_and(|value| !validate_order_key(value))
        || b.is_some_and(|value| !validate_order_key(value))
        || matches!((a, b), (Some(a), Some(b)) if a >= b)
    {
        return Err(());
    }

    if a.is_none() {
        let Some(b) = b else {
            return Ok("a0".to_owned());
        };
        let integer = get_integer_part(b)?;
        let fractional = &b[integer.len()..];
        if integer == format!("A{}", "0".repeat(26)) {
            return Ok(format!("{integer}{}", midpoint("", Some(fractional))?));
        }
        if integer < b {
            return Ok(integer.to_owned());
        }
        return decrement_integer(integer)?.ok_or(());
    }

    let a = a.ok_or(())?;
    if b.is_none() {
        let integer = get_integer_part(a)?;
        let fractional = &a[integer.len()..];
        return match increment_integer(integer)? {
            Some(next) => Ok(next),
            None => Ok(format!("{integer}{}", midpoint(fractional, None)?)),
        };
    }

    let b = b.ok_or(())?;
    let integer_a = get_integer_part(a)?;
    let fractional_a = &a[integer_a.len()..];
    let integer_b = get_integer_part(b)?;
    let fractional_b = &b[integer_b.len()..];
    if integer_a == integer_b {
        return Ok(format!(
            "{integer_a}{}",
            midpoint(fractional_a, Some(fractional_b))?
        ));
    }
    let incremented = increment_integer(integer_a)?.ok_or(())?;
    if incremented.as_str() < b {
        return Ok(incremented);
    }
    Ok(format!(
        "{integer_a}{}",
        midpoint(fractional_a, None)?
    ))
}

fn generate_n_keys_between_inner(
    a: Option<&str>,
    b: Option<&str>,
    n: usize,
) -> Result<Vec<String>, ()> {
    if n == 0 {
        return Ok(Vec::new());
    }
    if n == 1 {
        return Ok(vec![generate_key_between_inner(a, b)?]);
    }
    if b.is_none() {
        let mut current = generate_key_between_inner(a, None)?;
        let mut result = vec![current.clone()];
        for _ in 1..n {
            current = generate_key_between_inner(Some(&current), None)?;
            result.push(current.clone());
        }
        return Ok(result);
    }
    if a.is_none() {
        let mut current = generate_key_between_inner(None, b)?;
        let mut result = vec![current.clone()];
        for _ in 1..n {
            current = generate_key_between_inner(None, Some(&current))?;
            result.push(current.clone());
        }
        result.reverse();
        return Ok(result);
    }

    let middle = n / 2;
    let center = generate_key_between_inner(a, b)?;
    let mut result = generate_n_keys_between_inner(a, Some(&center), middle)?;
    result.push(center.clone());
    result.extend(generate_n_keys_between_inner(
        Some(&center),
        b,
        n - middle - 1,
    )?);
    Ok(result)
}

fn digit(index: usize) -> Result<u8, ()> {
    BASE_62_DIGITS.as_bytes().get(index).copied().ok_or(())
}

fn digit_index(value: u8) -> Option<usize> {
    BASE_62_DIGITS.as_bytes().iter().position(|digit| *digit == value)
}
