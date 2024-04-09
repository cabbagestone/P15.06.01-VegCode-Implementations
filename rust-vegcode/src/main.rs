use chrono::{prelude::NaiveDate, Duration};

fn main() {
    let vegcode = to_vegcode();
    // println!("Vegcode: {}", to_vegcode());
    // from_vegcode(String::from("0zHg7Yau0"));
    println!("Vegcode: {}", vegcode);
    from_vegcode(vegcode);
}

fn to_vegcode() -> String {
    let mut vegcode = String::new();

    let vegcode_version = String::from("0");
    vegcode.push_str(&vegcode_version);

    let flag_author = false;
    let flag_draft = false;
    let flag_archive = false;
    let sacsi_id = String::from("A21.01.01");
    let v2_version = String::from(""); //1.0.0");
    let created_date = String::from("2024-04-08");
    let last_updated_date = String::from(""); //2024-04-09");

    let mut flags = String::from("000000");
    let mut v2_base62 = String::new();
    let mut flag_v2_4 = false;

    set_binary_string_bit(&mut flags, 5, flag_author);
    set_binary_string_bit(&mut flags, 4, flag_draft);
    set_binary_string_bit(&mut flags, 3, flag_archive);

    let flag_versioned_file = v2_version.len() != 0;

    let has_created_date = created_date.len() > 0;
    let has_last_updated_date = last_updated_date.len() > 0;

    if !flag_versioned_file && has_last_updated_date {
        panic!("Modified date is set on an unversioned file");
    }

    if has_last_updated_date && !has_created_date {
        panic!("Modified date is set without a created date");
    }

    if flag_versioned_file {
        v2_base62 = base10_to_base62(convert_period_separated_string_to_base10(
            v2_version.clone(),
        ));
        flag_v2_4 = v2_base62.len() == 4;
    }
    let sacsi_base62 = base10_to_base62(sacsi_id_to_base10(sacsi_id.clone()));
    let flag_sacsi_5 = sacsi_base62.len() == 5;

    set_binary_string_bit(&mut flags, 2, flag_sacsi_5);
    set_binary_string_bit(&mut flags, 1, flag_v2_4);
    set_binary_string_bit(&mut flags, 0, flag_versioned_file);

    let flags_base64 = base2_to_friendly64(flags);
    vegcode.push_str(&flags_base64);
    vegcode.push_str(&sacsi_base62);

    if flag_versioned_file {
        vegcode.push_str(&v2_base62);
    }

    if !has_created_date {
        return vegcode;
    }

    let created_datetime = NaiveDate::parse_from_str(&created_date, "%Y-%m-%d").unwrap();
    let days_between_created_datetime_and_jan_1_2000 = created_datetime
        .signed_duration_since(
            NaiveDate::from_ymd_opt(2000, 1, 1)
                .ok_or("Invalid date")
                .unwrap(),
        )
        .num_days();

    let created_date_base62 = base10_to_base62(days_between_created_datetime_and_jan_1_2000 as u64);

    vegcode.push_str(&created_date_base62);

    if !has_last_updated_date {
        return vegcode;
    }

    let days_between_updated_datetime_and_created_datetime =
        NaiveDate::parse_from_str(&last_updated_date, "%Y-%m-%d")
            .unwrap()
            .signed_duration_since(created_datetime)
            .num_days();

    let last_updated_date_base62 =
        base10_to_base62(days_between_updated_datetime_and_created_datetime as u64);

    vegcode.push_str(&last_updated_date_base62);

    vegcode
}

fn from_vegcode(vegcode: String) {
    let mut vegcode_iterator = vegcode.chars();

    let vegcode_version = vegcode_iterator.next().unwrap().to_string();
    println!("Vegcode version: {}", vegcode_version);

    let flags_base64 = vegcode_iterator.next().unwrap().to_string();

    let binding = friendly64_to_binary_string(flags_base64);

    let binding = format!("{:0>6}", binding);
    let mut flags = binding.chars();

    let flag_versioned_file = flags.next().unwrap() == '1';
    let read_flag_v2_4 = flags.next().unwrap() == '1';
    let read_flag_sacsi_5 = flags.next().unwrap() == '1';
    let read_flag_archive = flags.next().unwrap() == '1';
    let read_flag_draft = flags.next().unwrap() == '1';
    let read_flag_author = flags.next().unwrap() == '1';

    println!("I'm the Author: {}", read_flag_author);
    println!("File is Draft: {}", read_flag_draft);
    println!("File is Archived: {}", read_flag_archive);

    let sacsi_length = if read_flag_sacsi_5 { 5 } else { 4 };
    let v2_length = if read_flag_v2_4 { 4 } else { 3 };

    let vegcode_vec: Vec<char> = vegcode_iterator.collect();
    let sacsi_base62 = vegcode_vec.iter().take(sacsi_length).collect();
    let sacsi_id = base62_to_sacsi_id(sacsi_base62);
    println!("Sacsi ID: {}", sacsi_id);

    let mut v2_version = String::new();
    let mut last_updated_date = 0;
    let created_date_base62;

    if !flag_versioned_file {
        created_date_base62 = vegcode_vec.iter().skip(sacsi_length).collect();
    } else {
        let v2_base62 = vegcode_vec
            .iter()
            .skip(sacsi_length)
            .take(v2_length)
            .collect();

        created_date_base62 = vegcode_vec
            .iter()
            .skip(sacsi_length + v2_length)
            .take(3)
            .collect();

        let last_updated_date_base62 = vegcode_vec
            .iter()
            .skip(sacsi_length + v2_length + 3)
            .collect();

        v2_version = base62_to_period_separated_string(v2_base62);
        println!("V2 Version: {}", v2_version);
        last_updated_date = base62_to_base10(last_updated_date_base62);
    }

    let created_date = base62_to_base10(created_date_base62);
    let has_created_date = created_date > 0;

    if !has_created_date {
        return;
    }

    let created_datetime = NaiveDate::from_ymd_opt(2000, 1, 1)
        .ok_or("Invalid date")
        .unwrap()
        + Duration::days(created_date as i64);

    let created_datetime_string = created_datetime.format("%Y-%m-%d").to_string();
    println!("Created date: {}", created_datetime_string);

    let last_updated_datetime = created_datetime + Duration::days(last_updated_date as i64);

    if !last_updated_date > 0 {
        return;
    }

    let last_updated_datetime_string = last_updated_datetime.format("%Y-%m-%d").to_string();

    println!("Last updated date: {}", last_updated_datetime_string);
}

fn base62_to_period_separated_string(string_base62: String) -> String {
    let num = base62_to_base10(string_base62);
    base10_to_period_separated_string(num)
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
    let first_char = (((num / 1_000_000) + 96) as u8 as char).to_ascii_uppercase();
    sacsi_id.push(first_char);
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
        if remainder < 10 {
            string.push('0');
        }
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
    if num == 0 {
        return "0".to_string();
    }
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
