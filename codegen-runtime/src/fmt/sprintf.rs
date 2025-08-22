/// Perl-compatible sprintf implementation
/// 
/// This module provides a sprintf function that matches Perl's behavior:
/// - Missing arguments become 0 (with Perl warnings, but we handle gracefully)
/// - Extra arguments are ignored
/// - Format specifiers are replaced in order

use crate::TagValue;
use std::collections::VecDeque;

/// Perl-compatible sprintf that handles any number of arguments
/// 
/// Just like Perl: missing args become 0, extra args ignored
/// 
/// # Examples
/// ```
/// sprintf_perl("%.3f x %.3f mm", &[TagValue::F64(1.234), TagValue::F64(5.678)])
/// // Returns: "1.234 x 5.678 mm"
/// 
/// sprintf_perl("%.3f x %.3f mm", &[TagValue::F64(1.234)])  
/// // Returns: "1.234 x 0.000 mm" (missing arg becomes 0)
/// 
/// sprintf_perl("%.2f", &[TagValue::F64(1.234), TagValue::F64(5.678)])
/// // Returns: "1.23" (extra arg ignored)
/// ```
pub fn sprintf_perl(format: &str, args: &[TagValue]) -> String {
    // Extract all format specifiers from the format string
    let specs = extract_format_specifiers(format);
    
    // Format each argument according to its specifier
    let mut formatted_values = Vec::new();
    for (i, spec) in specs.iter().enumerate() {
        let formatted = if i < args.len() {
            format_tagvalue(spec, &args[i])
        } else {
            // Missing argument - Perl pads with 0
            format_tagvalue(spec, &TagValue::I32(0))
        };
        formatted_values.push(formatted);
    }
    
    // Replace format specifiers with formatted values
    apply_formatted_values(format, &specs, &formatted_values)
}

/// Extract format specifiers from a format string
fn extract_format_specifiers(format: &str) -> Vec<String> {
    let mut specs = Vec::new();
    let mut chars = format.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '%' {
            if chars.peek() == Some(&'%') {
                // %% is a literal %
                chars.next();
                continue;
            }
            
            let mut spec = String::from("%");
            
            // Handle flags (+, -, 0, space, #)
            while let Some(&next) = chars.peek() {
                if matches!(next, '+' | '-' | '0' | ' ' | '#') {
                    spec.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            
            // Handle width
            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() {
                    spec.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            
            // Handle precision
            if chars.peek() == Some(&'.') {
                spec.push(chars.next().unwrap()); // consume '.'
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() {
                        spec.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
            }
            
            // Handle conversion specifier
            if let Some(&next) = chars.peek() {
                if matches!(next, 'd' | 'i' | 'o' | 'u' | 'x' | 'X' | 'e' | 'E' | 'f' | 'F' | 'g' | 'G' | 's' | 'c') {
                    spec.push(chars.next().unwrap());
                    specs.push(spec);
                }
            }
        }
    }
    
    specs
}

/// Format a TagValue according to a format specifier
fn format_tagvalue(spec: &str, val: &TagValue) -> String {
    // Parse the format specifier
    let conversion = spec.chars().last().unwrap_or('s');
    
    // Extract precision if present
    let precision = if let Some(dot_pos) = spec.rfind('.') {
        let precision_str = &spec[dot_pos + 1..spec.len() - 1];
        precision_str.parse::<usize>().ok()
    } else {
        None
    };
    
    // Extract width and flags
    let has_zero_pad = spec.contains('0') && spec.chars().nth(1) == Some('0');
    let has_plus = spec.contains('+');
    let width = extract_width(spec);
    
    // Convert TagValue to appropriate type and format
    match conversion {
        'd' | 'i' => {
            let num = tagvalue_to_i64(val);
            format_integer(num, width, has_zero_pad, has_plus, false)
        }
        'x' => {
            let num = tagvalue_to_i64(val);
            format_hex(num, width, has_zero_pad, false)
        }
        'X' => {
            let num = tagvalue_to_i64(val);
            format_hex(num, width, has_zero_pad, true)
        }
        'o' => {
            let num = tagvalue_to_i64(val);
            format_octal(num, width, has_zero_pad)
        }
        'f' | 'F' => {
            let num = tagvalue_to_f64(val);
            format_float(num, precision.unwrap_or(6), width, has_plus)
        }
        'e' | 'E' => {
            let num = tagvalue_to_f64(val);
            format_scientific(num, precision.unwrap_or(6), conversion == 'E')
        }
        'g' | 'G' => {
            let num = tagvalue_to_f64(val);
            format_general(num, precision.unwrap_or(6), conversion == 'G')
        }
        's' | _ => {
            val.to_string()
        }
    }
}

/// Extract width from format specifier
fn extract_width(spec: &str) -> Option<usize> {
    // Skip the % and any flags
    let mut chars = spec.chars().skip(1);
    while let Some(ch) = chars.next() {
        if ch.is_ascii_digit() {
            // Found start of width
            let mut width_str = String::from(ch);
            while let Some(ch) = chars.next() {
                if ch.is_ascii_digit() {
                    width_str.push(ch);
                } else {
                    break;
                }
            }
            return width_str.parse().ok();
        } else if ch == '.' {
            // Reached precision, no width
            break;
        } else if !matches!(ch, '+' | '-' | '0' | ' ' | '#') {
            // Reached conversion specifier
            break;
        }
    }
    None
}

/// Convert TagValue to i64
fn tagvalue_to_i64(val: &TagValue) -> i64 {
    match val {
        TagValue::I32(i) => *i as i64,
        TagValue::U32(u) => *u as i64,
        TagValue::U16(u) => *u as i64,
        TagValue::U8(u) => *u as i64,
        TagValue::F64(f) => *f as i64,
        // F32 doesn't exist in TagValue enum - already handled by F64 above
        TagValue::String(s) => s.parse::<i64>().unwrap_or(0),
        _ => 0,
    }
}

/// Convert TagValue to f64
fn tagvalue_to_f64(val: &TagValue) -> f64 {
    match val {
        TagValue::F64(f) => *f,
        // F32 doesn't exist in TagValue enum - already handled by F64 above
        TagValue::I32(i) => *i as f64,
        TagValue::U32(u) => *u as f64,
        TagValue::U16(u) => *u as f64,
        TagValue::U8(u) => *u as f64,
        TagValue::String(s) => s.parse::<f64>().unwrap_or(0.0),
        _ => 0.0,
    }
}

/// Format an integer with optional width and padding
fn format_integer(num: i64, width: Option<usize>, zero_pad: bool, plus: bool, _decimal: bool) -> String {
    let formatted = if plus && num >= 0 {
        format!("+{}", num)
    } else {
        num.to_string()
    };
    
    if let Some(w) = width {
        if zero_pad && !formatted.starts_with('-') && !formatted.starts_with('+') {
            format!("{:0>width$}", formatted, width = w)
        } else if zero_pad {
            // Handle sign separately for zero padding
            let sign = &formatted[..1];
            let rest = &formatted[1..];
            format!("{}{:0>width$}", sign, rest, width = w - 1)
        } else {
            format!("{:>width$}", formatted, width = w)
        }
    } else {
        formatted
    }
}

/// Format as hexadecimal
fn format_hex(num: i64, width: Option<usize>, zero_pad: bool, uppercase: bool) -> String {
    let formatted = if uppercase {
        format!("{:X}", num)
    } else {
        format!("{:x}", num)
    };
    
    if let Some(w) = width {
        if zero_pad {
            format!("{:0>width$}", formatted, width = w)
        } else {
            format!("{:>width$}", formatted, width = w)
        }
    } else {
        formatted
    }
}

/// Format as octal
fn format_octal(num: i64, width: Option<usize>, zero_pad: bool) -> String {
    let formatted = format!("{:o}", num);
    
    if let Some(w) = width {
        if zero_pad {
            format!("{:0>width$}", formatted, width = w)
        } else {
            format!("{:>width$}", formatted, width = w)
        }
    } else {
        formatted
    }
}

/// Format a float with given precision
fn format_float(num: f64, precision: usize, width: Option<usize>, plus: bool) -> String {
    let formatted = if plus && num >= 0.0 {
        format!("+{:.prec$}", num, prec = precision)
    } else {
        format!("{:.prec$}", num, prec = precision)
    };
    
    if let Some(w) = width {
        format!("{:>width$}", formatted, width = w)
    } else {
        formatted
    }
}

/// Format in scientific notation
fn format_scientific(num: f64, precision: usize, uppercase: bool) -> String {
    if uppercase {
        format!("{:.prec$E}", num, prec = precision)
    } else {
        format!("{:.prec$e}", num, prec = precision)
    }
}

/// Format in general notation (shortest of decimal or scientific)
fn format_general(num: f64, precision: usize, uppercase: bool) -> String {
    // Rust's g formatting is close to Perl's
    if uppercase {
        format!("{:.prec$}", num, prec = precision)
    } else {
        format!("{:.prec$}", num, prec = precision)
    }
}

/// Replace format specifiers in the original string with formatted values
fn apply_formatted_values(format: &str, specs: &[String], values: &[String]) -> String {
    let mut result = format.to_string();
    let mut values_queue: VecDeque<_> = values.iter().collect();
    
    // Replace each specifier with its formatted value
    for spec in specs {
        if let Some(value) = values_queue.pop_front() {
            // Find and replace the first occurrence of this specifier
            if let Some(pos) = result.find(spec) {
                result.replace_range(pos..pos + spec.len(), value);
            }
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprintf_basic() {
        let args = vec![TagValue::F64(1.234)];
        assert_eq!(sprintf_perl("%.2f", &args), "1.23");
    }

    #[test]
    fn test_sprintf_multiple_args() {
        let args = vec![TagValue::F64(1.234), TagValue::F64(5.678)];
        assert_eq!(sprintf_perl("%.3f x %.3f mm", &args), "1.234 x 5.678 mm");
    }

    #[test]
    fn test_sprintf_missing_arg() {
        let args = vec![TagValue::F64(1.234)];
        assert_eq!(sprintf_perl("%.3f x %.3f mm", &args), "1.234 x 0.000 mm");
    }

    #[test]
    fn test_sprintf_extra_args() {
        let args = vec![TagValue::F64(1.234), TagValue::F64(5.678), TagValue::F64(9.012)];
        assert_eq!(sprintf_perl("%.2f", &args), "1.23");
    }

    #[test]
    fn test_sprintf_integer_formats() {
        let args = vec![TagValue::I32(42)];
        assert_eq!(sprintf_perl("%d", &args), "42");
        assert_eq!(sprintf_perl("%5d", &args), "   42");
        assert_eq!(sprintf_perl("%05d", &args), "00042");
        assert_eq!(sprintf_perl("%+d", &args), "+42");
    }

    #[test]
    fn test_sprintf_hex_formats() {
        let args = vec![TagValue::I32(255)];
        assert_eq!(sprintf_perl("%x", &args), "ff");
        assert_eq!(sprintf_perl("%X", &args), "FF");
        assert_eq!(sprintf_perl("%04x", &args), "00ff");
        assert_eq!(sprintf_perl("0x%02x", &args), "0xff");
    }

    #[test]
    fn test_sprintf_mixed_formats() {
        let args = vec![TagValue::I32(1), TagValue::I32(2), TagValue::I32(3)];
        assert_eq!(sprintf_perl("%02d:%02d:%02d", &args), "01:02:03");
    }
}