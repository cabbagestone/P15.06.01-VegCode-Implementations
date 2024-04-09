use chrono::prelude::*;

fn main() {
    let vercode_version = String::from("0");

    let flag_author = false;
    let flag_draft = false;
    let flag_archive = false;

    let mut flag_v2_4 = false;

    let sacsi_id = String::from("Z99.99.99");
    let v2_version = String::from("99.99.99");
    let created_date = String::from("2024-04-08");
    let last_updated_date = String::from("2024-04-09");

    let mut vercode = String::from("0");
    let mut flags = String::from("000000");
    let mut v2_base62 = String::new();

    set_binary_string_bit(&mut flags, 0, flag_author);
    set_binary_string_bit(&mut flags, 1, flag_draft);
    set_binary_string_bit(&mut flags, 2, flag_archive);

    let flag_unversioned_file = v2_version.len() == 0;

    if flag_unversioned_file && last_updated_date.len() > 0 {
        panic!("Modified date is set on an unversioned file");
    }

    if !flag_unversioned_file {
        v2_base62 = base10_to_base62(convert_period_separated_string_to_base10(
            v2_version.clone(),
        ));
        flag_v2_4 = v2_base62.len() == 4;
    }
    let sacsi_base62 = base10_to_base62(sacsi_id_to_base10(sacsi_id.clone()));
    let flag_sacsi_5 = sacsi_base62.len() == 5;

    set_binary_string_bit(&mut flags, 3, flag_sacsi_5);
    set_binary_string_bit(&mut flags, 4, flag_v2_4);
    set_binary_string_bit(&mut flags, 5, flag_unversioned_file);

    let flags_base64 = base2_to_friendly64(flags);

    let created_datetime = NaiveDate::parse_from_str(&created_date, "%Y-%m-%d").unwrap();
    let last_updated_datetime = NaiveDate::parse_from_str(&last_updated_date, "%Y-%m-%d").unwrap();

    let days_between_created_datetime_and_jan_1_2000 = created_datetime
        .signed_duration_since(
            NaiveDate::from_ymd_opt(2000, 1, 1)
                .ok_or("Invalid date")
                .unwrap(),
        )
        .num_days();

    let days_between_updated_datetime_and_created_datetime = last_updated_datetime
        .signed_duration_since(created_datetime)
        .num_days();

    let created_date_base62 = base10_to_base62(days_between_created_datetime_and_jan_1_2000 as u64);
    let last_updated_date_base62 =
        base10_to_base62(days_between_updated_datetime_and_created_datetime as u64);

    vercode.push_str(&vercode_version);
    vercode.push_str(&flags_base64);
    vercode.push_str(&sacsi_base62);

    if !flag_unversioned_file {
        vercode.push_str(&v2_base62);
        vercode.push_str(&created_date_base62);
        vercode.push_str(&last_updated_date_base62);
    } else {
        vercode.push_str(&created_date_base62);
    }

    println!("{}", vercode);

    // now process the vercode to get the values back

    let vercode_version = vercode.chars().nth(0).unwrap().to_string();
    let flags_base64 = vercode.chars().skip(1).take(1).collect::<String>();

    let flags = friendly64_to_binary_string(flags_base64);

    let read_flag_author = flags.chars().nth(0).unwrap() == '1';
    let read_flag_draft = flags.chars().nth(1).unwrap() == '1';
    let read_flag_archive = flags.chars().nth(2).unwrap() == '1';
    let read_flag_sacsi_5 = flags.chars().nth(3).unwrap() == '1';
    let read_flag_v2_4 = flags.chars().nth(4).unwrap() == '1';
    let read_flag_unversioned_file = flags.chars().nth(5).unwrap() == '1';

    let sacsi_length = if read_flag_sacsi_5 { 5 } else { 4 };
    let v2_length = if read_flag_v2_4 { 4 } else { 0 };

    let sacsi_base62 = vercode
        .chars()
        .skip(2)
        .take(sacsi_length)
        .collect::<String>();

    let mut read_created_date_base62 = String::new();
    let mut read_last_updated_date_base62 = String::new();
    let mut read_v2_base62 = String::new();

    if read_flag_unversioned_file {
        read_created_date_base62 = vercode
            .chars()
            .skip(2 + sacsi_length)
            .take(4)
            .collect::<String>();
    } else {
        read_v2_base62 = vercode
            .chars()
            .skip(2 + sacsi_length)
            .take(v2_length)
            .collect::<String>();
        read_created_date_base62 = vercode
            .chars()
            .skip(2 + sacsi_length + v2_length)
            .take(4)
            .collect::<String>();
        read_last_updated_date_base62 = vercode
            .chars()
            .skip(2 + sacsi_length + v2_length + 4)
            .take(4)
            .collect::<String>();
    }


}

fn sacsi_id_to_base10(sacsi_id: String) -> u64 {
    let sacsi_id = sacsi_id.to_lowercase();

    let first_char = sacsi_id.chars().nth(0).unwrap() as u64 - 96;

    let first_char = first_char * 1_000_000;

    let rest_of_id = sacsi_id.chars().skip(1).collect::<String>();

    first_char + convert_period_separated_string_to_base10(rest_of_id)
}

fn convert_period_separated_string_to_base10(string: String) -> u64 {
    let string: Vec<&str> = string.split(".").collect();
    let mut base10 = 0;
    let mut power = 0;
    for s in string.iter().rev() {
        let num = s.parse::<u64>().unwrap();
        base10 += num * 10u64.pow(power);
        power += 2;
    }
    base10
}

fn base62_to_sacsi_id(string_base62: String) -> String {
    let num = base62_to_base10(string_base62);
    base10_to_sacsi_id(num)
}

fn base10_to_sacsi_id(num: u64) -> String {
    let mut sacsi_id = String::new();
    let first_char = (num / 1_000_000) + 96;
    sacsi_id.push(first_char as u8 as char);
    let rest_of_id = num % 1_000_000;
    sacsi_id.push_str(&base10_to_period_separated_string(rest_of_id));
    sacsi_id
}

fn base10_to_period_separated_string(num: u64) -> String {
    let mut string = String::new();
    let mut num = num;
    while num > 0 {
        let remainder = num % 100;
        string.push_str(&remainder.to_string());
        string.push('.');
        num = num / 100;
    }
    string.pop();
    string.chars().rev().collect()
}

fn base62_to_base10(string_base62: String) -> u64 {
    let mut num = 0;
    let mut power = 0;
    for c in string_base62.chars().rev() {
        let index = get_base62_char_index(c);
        num += index * 62u64.pow(power);
        power += 1;
    }
    num
}

fn get_base62_char_index(character: char) -> u64 {
    let base62_chars = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let index = base62_chars.find(character).unwrap();
    index as u64
}

fn friendly64_to_binary_string(string_friendly64: String) -> String {
    let mut binary_string = String::new();
    for c in string_friendly64.chars() {
        let num = get_friendly64_char_index(c);
        let binary = base10_to_base2(num);
        binary_string.push_str(&binary);
    }
    binary_string
}

fn get_friendly64_char_index(character: char) -> u64 {
    let friendly_base64_chars = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_";
    let index = friendly_base64_chars.find(character).unwrap();
    index as u64
}

fn base10_to_base2(num: u64) -> String {
    let mut binary = String::new();
    let mut num = num;
    while num > 0 {
        let remainder = num % 2;
        binary.push_str(&remainder.to_string());
        num = num / 2;
    }
    binary.chars().rev().collect()
}

fn set_binary_string_bit(string: &mut String, index: usize, bit: bool) {
    match bit {
        true => set_char_for_string(string, index, '1'),
        false => set_char_for_string(string, index, '0'),
    }
}

fn set_char_for_string(string: &mut String, index: usize, character: char) {
    let mut chars: Vec<char> = string.chars().collect();
    chars[index] = character;
    *string = chars.into_iter().collect();
}

fn base2_to_friendly64(string_binary: String) -> String {
    let num = base2_to_base10(string_binary);
    base10_to_friendly64(num)
}

fn base10_to_base62(num: u64) -> String {
    let mut base62 = String::new();
    let mut num = num;
    while num > 0 {
        let remainder = num % 62;
        base62.push(get_base62_char(remainder));
        num = num / 62;
    }
    base62.chars().rev().collect()
}

fn base2_to_base10(string_binary: String) -> u64 {
    let mut num = 0;
    let mut power = 0;
    for c in string_binary.chars().rev() {
        if c == '1' {
            num += 2u64.pow(power);
        }
        power += 1;
    }
    num
}

fn base10_to_friendly64(num: u64) -> String {
    let mut base64 = String::new();
    let mut num = num;
    while num > 0 {
        let remainder = num % 64;
        base64.push(get_friendly64_char(remainder));
        num = num / 64;
    }
    base64.chars().rev().collect()
}

fn get_base62_char(num: u64) -> char {
    let base62_chars = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let base62_chars_vec: Vec<char> = base62_chars.chars().collect();
    base62_chars_vec[num as usize]
}

fn get_friendly64_char(num: u64) -> char {
    let friendly_base64_chars = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_";
    let friendly_base64_chars_vec: Vec<char> = friendly_base64_chars.chars().collect();
    friendly_base64_chars_vec[num as usize]
}
